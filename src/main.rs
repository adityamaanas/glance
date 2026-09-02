//! glance: a live orientation panel for one Claude Code session, meant for a herdr split pane.

mod herdr;
mod setup;
mod summary;
mod transcript;
mod view;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io::stdout;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use summary::{Cache, Summary};
use transcript::Transcript;
use view::Focus;

#[derive(Parser)]
#[command(
    name = "glance-panel",
    version,
    about = "glance: live orientation panel for a Claude Code session"
)]
struct Cli {
    /// herdr pane id of the Claude Code session to follow (default: the pane next to this one).
    #[arg(long)]
    pane: Option<String>,
    /// Claude Code session id to follow directly (no herdr needed).
    #[arg(long)]
    session: Option<String>,
    /// Never call the model; show free fields and the cached summary only.
    #[arg(long)]
    no_model: bool,
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// From inside a Claude Code pane: split right and start the panel there.
    Attach {
        /// Fraction of the width the new pane takes.
        #[arg(long, default_value_t = 0.3)]
        ratio: f64,
        /// Split even if the tab already has more than one pane.
        #[arg(long)]
        force: bool,
    },
    /// Print the summary JSON for a session and exit (development aid).
    Summarize {
        #[arg(long)]
        session: String,
    },
    /// Claude Code SessionStart hook entry point (reads the hook JSON on stdin, always exits 0).
    Hook {
        /// Register this binary as the SessionStart hook in ~/.claude/settings.json.
        #[arg(long)]
        install: bool,
        /// Remove the glance SessionStart hook from ~/.claude/settings.json.
        #[arg(long)]
        uninstall: bool,
    },
}

enum Msg {
    Grew,
    Status(String),
    Key(KeyCode, KeyModifiers),
    Tick,
    Summary(Result<Summary>, usize),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Cmd::Attach { ratio, force }) => attach(ratio, force),
        Some(Cmd::Summarize { session }) => summarize_once(&session),
        Some(Cmd::Hook { install: true, .. }) => {
            println!("{}", setup::install_hook()?);
            Ok(())
        }
        Some(Cmd::Hook {
            uninstall: true, ..
        }) => {
            println!("{}", setup::uninstall_hook()?);
            Ok(())
        }
        Some(Cmd::Hook { .. }) => hook_main(),
        None => run(cli),
    }
}

/// Runs as the SessionStart hook: skip subagents, non-herdr panes and `claude -p`, then attach.
fn hook_main() -> Result<()> {
    let mut input = String::new();
    let _ = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input);
    let decision = if input.contains("\"agent_id\"") {
        "skip: subagent".to_string()
    } else if std::env::var("HERDR_PANE_ID").is_err() {
        "skip: not in herdr".to_string()
    } else if let Some(cmd) = print_mode_ancestor() {
        format!("skip: print mode ({cmd})")
    } else {
        match attach(0.3, false) {
            Ok(()) => "attach: ok".to_string(),
            Err(e) => format!("attach failed: {e}"),
        }
    };
    hook_log(&decision);
    Ok(())
}

fn hook_log(line: &str) {
    if let Ok(dir) = summary::state_dir() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("hook.log"))
        {
            let pane = std::env::var("HERDR_PANE_ID").unwrap_or_else(|_| "none".into());
            let _ = writeln!(f, "{} pane={pane}: {line}", summary::now_secs());
        }
    }
}

/// Walk up the process tree to the claude process; return its command line if it is a `-p` run.
#[cfg(unix)]
fn print_mode_ancestor() -> Option<String> {
    let mut pid = std::process::id();
    for _ in 0..8 {
        let out = std::process::Command::new("ps")
            .args(["-o", "ppid=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        let ppid: u32 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        if ppid <= 1 {
            return None;
        }
        let out = std::process::Command::new("ps")
            .args(["-o", "command=", "-p", &ppid.to_string()])
            .output()
            .ok()?;
        let cmd = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if cmd.contains("claude") {
            let padded = format!(" {cmd} ");
            return (padded.contains(" -p ") || padded.contains(" --print ")).then_some(cmd);
        }
        pid = ppid;
    }
    None
}

#[cfg(not(unix))]
fn print_mode_ancestor() -> Option<String> {
    None
}

fn attach(ratio: f64, force: bool) -> Result<()> {
    let pane = std::env::var("HERDR_PANE_ID")
        .context("HERDR_PANE_ID not set; run this inside a herdr pane")?;
    let client = herdr::Client::from_env().ok_or_else(|| anyhow!("herdr socket not found"))?;
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let command = format!("{exe} --pane {pane}");
    let others: Vec<String> = client
        .tab_panes(&pane)?
        .into_iter()
        .filter(|p| p != &pane)
        .collect();

    // Already attached: a sibling pane is running glance.
    // Idle sibling shell (what herdr leaves after a restart): reuse it instead of splitting again.
    let mut idle_shell: Option<String> = None;
    for other in &others {
        let names = client.foreground_names(other).unwrap_or_default();
        if names.iter().any(|n| is_self(n)) {
            println!("glance: already running in pane {other}");
            return Ok(());
        }
        if is_idle_shell(&names) && idle_shell.is_none() {
            idle_shell = Some(other.clone());
        }
    }
    let target = match idle_shell {
        Some(t) => t,
        None => {
            if !others.is_empty() && !force {
                eprintln!("glance: tab already has other panes; use --force to split anyway");
                return Ok(());
            }
            let cwd = std::env::current_dir()?.to_string_lossy().to_string();
            // herdr's ratio is the share the original pane keeps, so invert it.
            herdr::split_right(&pane, 1.0 - ratio, &cwd)?
        }
    };
    start_in_pane(&client, &target, &command)?;
    println!("glance: panel started in pane {target}");
    Ok(())
}

/// Whether a process name is this binary (whatever it was installed as).
fn is_self(name: &str) -> bool {
    let me = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "glance-panel".to_string());
    name == me || name == "glance" || name == "glance-panel"
}

/// The foreground group of a pane whose shell sits at its prompt is just the shell itself.
fn is_idle_shell(names: &[String]) -> bool {
    !names.is_empty()
        && names.iter().all(|n| {
            matches!(
                n.trim_start_matches('-'),
                "zsh" | "bash" | "fish" | "sh" | "nu" | "login"
            )
        })
}

/// Wait for the pane's shell prompt, type the command, and confirm glance came up (one retry).
fn start_in_pane(client: &herdr::Client, pane: &str, command: &str) -> Result<()> {
    let ready_by = Instant::now() + Duration::from_secs(15);
    while Instant::now() < ready_by {
        if is_idle_shell(&client.foreground_names(pane).unwrap_or_default()) {
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    for _ in 0..2 {
        herdr::run_in_pane(pane, command)?;
        let up_by = Instant::now() + Duration::from_secs(5);
        while Instant::now() < up_by {
            thread::sleep(Duration::from_millis(250));
            let names = client.foreground_names(pane).unwrap_or_default();
            if names.iter().any(|n| is_self(n)) {
                return Ok(());
            }
        }
    }
    bail!("glance did not start in pane {pane}")
}

fn summarize_once(session: &str) -> Result<()> {
    let path = transcript::find_transcript(session)?;
    let mut tr = Transcript::open(&path);
    tr.read_new()?;
    let text = summary::pending_text(&tr, 0);
    let s = summary::summarize(&Summary::default(), &text, tr.free.title.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&s)?);
    // Seed the cache so a panel opened on this session starts from this pass.
    let cache = Cache {
        version: summary::CACHE_VERSION,
        summary: s,
        turns_done: tr.turns.len(),
        updated_at: summary::now_secs(),
        source: summary::model(),
    };
    summary::save_cache(session, &cache)?;
    Ok(())
}

struct App {
    session_id: String,
    client: Option<herdr::Client>,
    pane: Option<String>,
    watched_path: Arc<Mutex<std::path::PathBuf>>,
    tr: Transcript,
    cache: Cache,
    agent_status: Option<String>,
    analyzing: bool,
    dirty: bool,
    last_growth: Option<Instant>,
    error: Option<String>,
    scroll: u16,
    no_model: bool,
    offer: bool,
    /// User-chosen focus; None follows the model's `focus`.
    focus_override: Option<Focus>,
    pinned: bool,
    rail: bool,
    tx: Sender<Msg>,
}

impl App {
    fn focus(&self) -> Focus {
        if !self.cache.summary.is_multi() {
            return Focus::All;
        }
        if self.pinned {
            return self.focus_override.clone().unwrap_or(Focus::All);
        }
        match self.cache.summary.focus.as_str() {
            "trunk" => Focus::Trunk,
            id => Focus::Branch(id.to_string()),
        }
    }

    fn focus_order(&self) -> Vec<Focus> {
        let mut v = vec![Focus::All, Focus::Trunk];
        v.extend(
            self.cache
                .summary
                .branches
                .iter()
                .map(|b| Focus::Branch(b.id.clone())),
        );
        v
    }

    fn cycle_focus(&mut self, step: isize) {
        let order = self.focus_order();
        let cur = self.focus();
        let idx = order.iter().position(|f| *f == cur).unwrap_or(0) as isize;
        let next = (idx + step).rem_euclid(order.len() as isize) as usize;
        self.focus_override = Some(order[next].clone());
        self.pinned = true;
    }
}

fn run(cli: Cli) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Msg>();
    let client = herdr::Client::from_env();

    // Resolve which session to follow.
    let mut agent_status = None;
    let mut watched_pane = None;
    let mut agent_cwd: Option<String> = None;
    let session_id = if let Some(s) = cli.session {
        s
    } else {
        let client = client
            .as_ref()
            .ok_or_else(|| anyhow!("no herdr socket; pass --session <id>"))?;
        let pane = match cli.pane {
            Some(p) => p,
            None => neighbor_pane(client)?,
        };
        let info = wait_for_session(client, &pane)?;
        agent_status = Some(info.agent_status.clone());
        agent_cwd = info.cwd.clone();
        client.watch_status(pane.clone(), status_forwarder(tx.clone()));
        watched_pane = Some(pane.clone());
        info.agent_session.map(|s| s.value).ok_or_else(|| {
            anyhow!("pane {pane} has no Claude session yet (is Claude Code running there?)")
        })?
    };

    // A fresh session has no transcript until its first prompt; watch the path it will get.
    let path = match transcript::find_transcript(&session_id) {
        Ok(p) => p,
        Err(e) => match &agent_cwd {
            Some(cwd) => transcript::expected_path(cwd, &session_id)?,
            None => return Err(e),
        },
    };
    let mut tr = Transcript::open(&path);
    tr.read_new()?;
    let cache = summary::load_cache(&session_id).unwrap_or_else(|| Cache {
        version: summary::CACHE_VERSION,
        summary: summary::heuristic(&tr),
        turns_done: 0,
        updated_at: 0,
        source: "heuristic".into(),
    });

    let watched_path = Arc::new(Mutex::new(tr.path.clone()));
    let mut app = App {
        session_id,
        client,
        pane: watched_pane,
        watched_path: watched_path.clone(),
        tr,
        cache,
        agent_status,
        analyzing: false,
        dirty: true,
        last_growth: Some(Instant::now() - Duration::from_secs(10)),
        error: None,
        scroll: 0,
        no_model: cli.no_model,
        offer: setup::should_offer(),
        focus_override: None,
        pinned: false,
        rail: false,
        tx: tx.clone(),
    };

    spawn_poller(watched_path, tx.clone());
    spawn_input(tx.clone());

    let mut terminal = init_terminal()?;
    let result = event_loop(&mut app, &rx, &mut terminal);
    restore_terminal();
    result
}

fn status_forwarder(tx: Sender<Msg>) -> Sender<String> {
    let (stx, srx) = mpsc::channel::<String>();
    thread::spawn(move || {
        for s in srx {
            if tx.send(Msg::Status(s)).is_err() {
                break;
            }
        }
    });
    stx
}

/// The Claude pane is whichever agent pane shares this pane's tab.
fn neighbor_pane(client: &herdr::Client) -> Result<String> {
    let tab = std::env::var("HERDR_TAB_ID").ok();
    let me = std::env::var("HERDR_PANE_ID").ok();
    let result = client.call("agent.list", serde_json::json!({}))?;
    let agents = result
        .get("agents")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    for a in agents {
        let pane = a.get("pane_id").and_then(|v| v.as_str()).unwrap_or("");
        let in_tab = a.get("tab_id").and_then(|v| v.as_str()) == tab.as_deref();
        if in_tab && Some(pane) != me.as_deref() {
            return Ok(pane.to_string());
        }
    }
    bail!("no Claude Code pane found in this tab; pass --pane <id>")
}

/// A fresh Claude pane may not have reported its session yet; give it a moment.
fn wait_for_session(client: &herdr::Client, pane: &str) -> Result<herdr::AgentInfo> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let info = client.agent_get(pane)?;
        if info.agent_session.is_some() || Instant::now() >= deadline {
            return Ok(info);
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn spawn_poller(path: Arc<Mutex<std::path::PathBuf>>, tx: Sender<Msg>) {
    thread::spawn(move || {
        let mut last_len = 0u64;
        loop {
            thread::sleep(Duration::from_millis(700));
            let p = path.lock().map(|g| g.clone()).unwrap_or_default();
            let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(last_len);
            if len != last_len {
                last_len = len;
                if tx.send(Msg::Grew).is_err() {
                    return;
                }
            }
        }
    });
}

fn spawn_input(tx: Sender<Msg>) {
    thread::spawn(move || loop {
        match event::poll(Duration::from_millis(500)) {
            Ok(true) => {
                if let Ok(Event::Key(k)) = event::read() {
                    if tx.send(Msg::Key(k.code, k.modifiers)).is_err() {
                        return;
                    }
                }
            }
            Ok(false) => {
                if tx.send(Msg::Tick).is_err() {
                    return;
                }
            }
            Err(_) => return,
        }
    });
}

type Term = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

fn init_terminal() -> Result<Term> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
    Ok(ratatui::Terminal::new(
        ratatui::backend::CrosstermBackend::new(stdout()),
    )?)
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
}

fn event_loop(app: &mut App, rx: &Receiver<Msg>, terminal: &mut Term) -> Result<()> {
    draw(app, terminal)?;
    loop {
        let msg = rx.recv_timeout(Duration::from_secs(1)).unwrap_or(Msg::Tick);
        match msg {
            Msg::Grew => {
                if let Err(e) = app.tr.read_new() {
                    app.error = Some(e.to_string());
                }
                app.dirty = true;
                app.last_growth = Some(Instant::now());
            }
            Msg::Status(s) => {
                app.agent_status = Some(s);
                follow_session_change(app);
            }
            Msg::Key(code, _)
                if app.offer && matches!(code, KeyCode::Char('y') | KeyCode::Char('Y')) =>
            {
                app.offer = false;
                match setup::install_hook() {
                    Ok(_) => {
                        let _ = setup::record_offer("accepted");
                        app.error = None;
                    }
                    Err(e) => app.error = Some(format!("hook install failed: {e}")),
                }
            }
            Msg::Key(code, _)
                if app.offer && matches!(code, KeyCode::Char('n') | KeyCode::Char('N')) =>
            {
                app.offer = false;
                let _ = setup::record_offer("declined");
            }
            Msg::Key(code, mods) => match code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Char('r') => {
                    app.dirty = true;
                    app.last_growth = Some(Instant::now() - Duration::from_secs(10));
                    app.cache.turns_done = app
                        .cache
                        .turns_done
                        .min(app.tr.turns.len().saturating_sub(1));
                }
                KeyCode::Down | KeyCode::Char('j') => app.scroll = app.scroll.saturating_add(1),
                KeyCode::Up | KeyCode::Char('k') => app.scroll = app.scroll.saturating_sub(1),
                KeyCode::Char('v') => {
                    app.rail = !app.rail;
                    app.scroll = 0;
                }
                KeyCode::Char(']') | KeyCode::Right => app.cycle_focus(1),
                KeyCode::Char('[') | KeyCode::Left => app.cycle_focus(-1),
                KeyCode::Char('0') => {
                    app.focus_override = Some(Focus::All);
                    app.pinned = true;
                }
                KeyCode::Char('p') => {
                    app.pinned = !app.pinned;
                    if !app.pinned {
                        app.focus_override = None;
                    }
                }
                _ => {}
            },
            Msg::Tick => {}
            Msg::Summary(result, turns_done) => {
                app.analyzing = false;
                match result {
                    Ok(s) => {
                        app.cache = Cache {
                            version: summary::CACHE_VERSION,
                            summary: s,
                            turns_done,
                            updated_at: summary::now_secs(),
                            source: summary::model(),
                        };
                        app.error = None;
                        if let Err(e) = summary::save_cache(&app.session_id, &app.cache) {
                            app.error = Some(e.to_string());
                        }
                    }
                    Err(e) => app.error = Some(e.to_string()),
                }
            }
        }
        maybe_summarize(app);
        draw(app, terminal)?;
    }
}

/// If the watched pane now hosts a different Claude session (after /clear or a resume), switch to it.
fn follow_session_change(app: &mut App) {
    let (Some(client), Some(pane)) = (app.client.as_ref(), app.pane.as_ref()) else {
        return;
    };
    let Ok(info) = client.agent_get(pane) else {
        return;
    };
    let Some(sid) = info.agent_session.map(|s| s.value) else {
        return;
    };
    if sid == app.session_id {
        return;
    }
    let Ok(path) = transcript::find_transcript(&sid) else {
        return;
    };
    let mut tr = Transcript::open(&path);
    if tr.read_new().is_err() {
        return;
    }
    app.cache = summary::load_cache(&sid).unwrap_or_else(|| Cache {
        version: summary::CACHE_VERSION,
        summary: summary::heuristic(&tr),
        turns_done: 0,
        updated_at: 0,
        source: "heuristic".into(),
    });
    if let Ok(mut guard) = app.watched_path.lock() {
        *guard = path;
    }
    app.tr = tr;
    app.session_id = sid;
    app.dirty = true;
    app.scroll = 0;
    app.error = None;
    app.last_growth = Some(Instant::now());
}

/// Start a model pass when there are unsummarized turns, the session is not mid-turn, and growth has settled.
fn maybe_summarize(app: &mut App) {
    if app.no_model || app.analyzing || !app.dirty {
        return;
    }
    if app.cache.turns_done >= app.tr.turns.len() {
        app.dirty = false;
        return;
    }
    if app.agent_status.as_deref() == Some("working") {
        return;
    }
    let settled = app
        .last_growth
        .map(|t| t.elapsed() >= Duration::from_secs(2))
        .unwrap_or(true);
    if !settled {
        return;
    }
    let text = summary::pending_text(&app.tr, app.cache.turns_done);
    let prev = app.cache.summary.clone();
    let title = app
        .tr
        .free
        .custom_title
        .clone()
        .or_else(|| app.tr.free.title.clone());
    let turns_done = app.tr.turns.len();
    let tx = app.tx.clone();
    app.analyzing = true;
    app.dirty = false;
    thread::spawn(move || {
        let result = summary::summarize(&prev, &text, title.as_deref());
        let _ = tx.send(Msg::Summary(result, turns_done));
    });
}

fn draw(app: &App, terminal: &mut Term) -> Result<()> {
    let state = view::ViewState {
        free: &app.tr.free,
        summary: &app.cache.summary,
        agent_status: app.agent_status.as_deref(),
        source: &app.cache.source,
        updated_at: app.cache.updated_at,
        analyzing: app.analyzing,
        error: app.error.as_deref(),
        scroll: app.scroll,
        waiting: !app.tr.path.exists(),
        offer: app.offer,
        focus: app.focus(),
        pinned: app.pinned,
        rail: app.rail,
    };
    terminal.draw(|f| view::draw(f, &state))?;
    Ok(())
}
