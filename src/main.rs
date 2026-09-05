//! glance: a live orientation panel for one Claude Code session, meant for a herdr split pane.

mod evidence;
mod herdr;
mod setup;
mod summary;
mod todos;
mod transcript;
mod transport;
mod view;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEvent,
    MouseEventKind,
};
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
    /// Summary model (overrides GLANCE_MODEL and config.json).
    #[arg(long, global = true)]
    model: Option<String>,
    /// Minimum seconds between summary calls (default 30).
    #[arg(long, global = true)]
    refresh_seconds: Option<u64>,
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Manage personal reminders (defaults to this herdr pane's session).
    Todo {
        /// Text to append; omit to list todos.
        text: Option<String>,
        #[arg(long)]
        session: Option<String>,
        /// Todo ID whose status should change.
        #[arg(long, requires = "status", conflicts_with_all = ["text", "delete", "carry_from"])]
        set: Option<String>,
        #[arg(long, value_enum, requires = "set")]
        status: Option<todos::Status>,
        #[arg(long, conflicts_with_all = ["text", "set", "carry_from"])]
        delete: Option<String>,
        /// Explicitly copy reminders from another session as pending items.
        #[arg(long, conflicts_with_all = ["text", "set", "delete"])]
        carry_from: Option<String>,
    },
    /// Show cached item relationships, or export a standalone HTML graph.
    Graph {
        #[arg(long)]
        session: String,
        #[arg(long, num_args = 0..=1, default_missing_value = "glance-graph.html")]
        html: Option<std::path::PathBuf>,
        #[arg(long, requires = "html")]
        open: bool,
    },
    /// Remove old summary caches; personal configuration and todos are preserved.
    CacheClean {
        #[arg(long, default_value_t = 30)]
        older_than_days: u64,
        #[arg(long)]
        dry_run: bool,
    },
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
    Mouse(MouseEvent),
    Summary(Box<Result<Summary>>, usize, u64, Vec<todos::Todo>),
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    let cfg = if matches!(
        cli.command,
        Some(Cmd::Hook {
            install: false,
            uninstall: false
        })
    ) {
        setup::Config::default()
    } else {
        setup::read_config()?
    };
    summary::configure(cli.model.as_deref(), &cfg);
    cli.no_model |= cfg.no_model;
    cli.refresh_seconds = cli.refresh_seconds.or(cfg.refresh_seconds);
    if cli.command.is_none() {
        if let Some(days) = cfg.cache_retention_days {
            summary::clean_cache(days, false)?;
        }
    }
    match cli.command {
        Some(Cmd::Todo {
            text,
            session,
            set,
            status,
            delete,
            carry_from,
        }) => todo_command(
            session.or(cli.session),
            text,
            set.zip(status),
            delete,
            carry_from,
        ),
        Some(Cmd::Graph {
            session,
            html,
            open,
        }) => export_graph(&session, html, open),
        Some(Cmd::CacheClean {
            older_than_days,
            dry_run,
        }) => {
            for path in summary::clean_cache(older_than_days, dry_run)? {
                println!("{}", path.display());
            }
            Ok(())
        }
        Some(Cmd::Attach { ratio, force }) => attach(ratio, force),
        Some(Cmd::Summarize { session }) => {
            if cli.no_model {
                bail!("model calls disabled by --no-model or config.json");
            }
            summarize_once(&session)
        }
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
    let hook_input: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let decision = if hook_input
        .get("agent_id")
        .and_then(serde_json::Value::as_str)
        .is_some()
    {
        "skip: subagent".to_string()
    } else if std::env::var("HERDR_PANE_ID").is_err() {
        "skip: not in herdr".to_string()
    } else if print_mode_ancestor().is_some() {
        "skip: print mode".to_string()
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
    if !ratio.is_finite() || ratio <= 0.0 || ratio >= 1.0 {
        bail!("ratio must be greater than 0 and less than 1");
    }
    let pane = std::env::var("HERDR_PANE_ID")
        .context("HERDR_PANE_ID not set; run this inside a herdr pane")?;
    let client = herdr::Client::from_env().ok_or_else(|| anyhow!("herdr socket not found"))?;
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let command = format!(
        "{} --pane {}",
        shell_words::quote(&exe),
        shell_words::quote(&pane)
    );
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
    if !is_idle_shell(&client.foreground_names(pane).unwrap_or_default()) {
        bail!("pane {pane} did not become an idle shell");
    }
    for _ in 0..2 {
        if !is_idle_shell(&client.foreground_names(pane).unwrap_or_default()) {
            bail!("pane {pane} is no longer an idle shell");
        }
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
    let mut s = Summary::default();
    let mut from = 0;
    while from < tr.turns.len() {
        let (text, end) = summary::pending_chunk(&tr, from);
        eprintln!(
            "Summarizing turns {}–{} of {}",
            from + 1,
            end,
            tr.turns.len()
        );
        let todo_path = todos::path(session)?;
        let snapshot = todos::load(&todo_path)?;
        s = summary::summarize(&s, &text, tr.free.title.as_deref(), &snapshot.items)?;
        if !s.todo_updates.is_empty() {
            todos::edit(&todo_path, |store| {
                store.apply(&snapshot.items, &s.todo_updates, &tr, end);
                Ok(())
            })?;
        }
        s.todo_updates.clear();
        evidence::normalize(&mut s, end);
        from = end;
    }
    println!("{}", serde_json::to_string_pretty(&s)?);
    // Seed the cache so a panel opened on this session starts from this pass.
    let cache = Cache {
        version: summary::CACHE_VERSION,
        summary: s,
        turns_done: tr.turns.len(),
        fingerprint: tr.fingerprint(tr.turns.len()),
        updated_at: summary::now_secs(),
        source: summary::model(),
    };
    summary::save_cache(session, &cache)?;
    Ok(())
}

fn todo_command(
    session: Option<String>,
    text: Option<String>,
    set: Option<(String, todos::Status)>,
    delete: Option<String>,
    carry: Option<String>,
) -> Result<()> {
    let session = match session {
        Some(id) => id,
        None => {
            let client =
                herdr::Client::from_env().context("pass todo --session <id> outside herdr")?;
            let pane = std::env::var("HERDR_PANE_ID").context("no herdr pane; pass --session")?;
            client
                .agent_get(&pane)?
                .agent_session
                .context("pane has no session")?
                .value
        }
    };
    let path = todos::path(&session)?;
    let mut tr = transcript::find_transcript(&session)
        .ok()
        .map(|p| Transcript::open(&p));
    if let Some(tr) = &mut tr {
        tr.read_new()?;
    }
    let turns = tr.as_ref().map_or(0, |tr| tr.turns.len());
    let fingerprint = tr.as_ref().map_or_else(
        || Transcript::open(std::path::Path::new("unused")).fingerprint(0),
        |tr| tr.fingerprint(turns),
    );
    let store = if text.is_some() || set.is_some() || delete.is_some() || carry.is_some() {
        let source = carry
            .map(|id| {
                if id == session {
                    bail!("cannot carry todos into the same session");
                }
                todos::load(&todos::path(&id)?)
            })
            .transpose()?;
        todos::edit(&path, |s| {
            if let Some(text) = text {
                s.add(&text, turns, fingerprint)?;
            }
            if let Some((id, status)) = set {
                s.set(&id, status, turns, fingerprint)?;
            }
            if let Some(id) = delete {
                s.delete(&id)?;
            }
            if let Some(source) = source {
                for item in source.items {
                    s.add(&item.text, turns, fingerprint)?;
                }
            }
            Ok(())
        })?
    } else {
        todos::load(&path)?
    };
    println!("{}", serde_json::to_string_pretty(&store.items)?);
    Ok(())
}

fn export_graph(session: &str, path: Option<std::path::PathBuf>, launch: bool) -> Result<()> {
    let mut tr = Transcript::open(&transcript::find_transcript(session)?);
    tr.read_new()?;
    let mut cache =
        summary::load_cache(session).context("no compatible summary cache; run summarize first")?;
    if cache.turns_done > tr.turns.len() || cache.fingerprint != tr.fingerprint(cache.turns_done) {
        bail!("summary cache belongs to a different transcript revision; refresh it first");
    }
    evidence::normalize(&mut cache.summary, cache.turns_done);
    if let Some(path) = path {
        let path = std::path::absolute(path)?;
        setup::atomic_write(&path, evidence::html(&cache.summary, &tr)?.as_bytes())?;
        println!("{}", path.display());
        if launch {
            open::that(&path)?;
        }
    } else {
        let nodes = evidence::nodes(&cache.summary);
        for (idx, depth) in evidence::graph_rows(&nodes) {
            let n = &nodes[idx];
            println!(
                "{}{} [{}] {}",
                "  ".repeat(depth),
                if depth == 0 { "●" } else { "└─" },
                n.id,
                n.text
            );
        }
    }
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
    generation: u64,
    refresh_seconds: u64,
    last_summary_started: Option<Instant>,
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
    inspect: bool,
    graph: bool,
    selection: usize,
    tx: Sender<Msg>,
}

impl App {
    fn finish_summary(
        &mut self,
        result: Result<Summary>,
        turns_done: usize,
        generation: u64,
    ) -> bool {
        if self.generation != generation {
            return false;
        }
        self.analyzing = false;
        match result {
            Ok(mut summary) => {
                evidence::normalize(&mut summary, turns_done);
                self.cache = Cache {
                    version: summary::CACHE_VERSION,
                    summary,
                    turns_done,
                    fingerprint: self.tr.fingerprint(turns_done),
                    updated_at: summary::now_secs(),
                    source: summary::model(),
                };
                self.error = None;
                self.dirty |= turns_done < self.tr.turns.len();
                true
            }
            Err(e) => {
                self.error = Some(e.to_string());
                false
            }
        }
    }

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
    let cache = summary::load_cache(&session_id)
        .filter(|c| c.turns_done <= tr.turns.len() && c.fingerprint == tr.fingerprint(c.turns_done))
        .unwrap_or_else(|| Cache {
            version: summary::CACHE_VERSION,
            summary: summary::heuristic(&tr),
            turns_done: 0,
            fingerprint: 0,
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
        generation: 0,
        refresh_seconds: cli.refresh_seconds.unwrap_or(30),
        last_summary_started: None,
        dirty: true,
        last_growth: Some(Instant::now() - Duration::from_secs(10)),
        error: None,
        scroll: 0,
        no_model: cli.no_model,
        offer: setup::should_offer(),
        focus_override: None,
        pinned: false,
        rail: false,
        inspect: false,
        graph: false,
        selection: 0,
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
        let mut last = None;
        loop {
            thread::sleep(Duration::from_millis(700));
            let p = path.lock().map(|g| g.clone()).unwrap_or_default();
            let signature = std::fs::metadata(&p)
                .ok()
                .map(|m| (p, m.len(), m.modified().ok()));
            if signature != last {
                last = signature;
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
                let msg = match event::read() {
                    Ok(Event::Key(k)) if k.kind != event::KeyEventKind::Release => {
                        Msg::Key(k.code, k.modifiers)
                    }
                    Ok(Event::Mouse(m)) => Msg::Mouse(m),
                    _ => Msg::Tick,
                };
                if tx.send(msg).is_err() {
                    return;
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
    stdout().execute(EnableMouseCapture)?;
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
    let _ = stdout().execute(DisableMouseCapture);
}

fn event_loop(app: &mut App, rx: &Receiver<Msg>, terminal: &mut Term) -> Result<()> {
    draw(app, terminal)?;
    loop {
        let msg = rx.recv_timeout(Duration::from_secs(1)).unwrap_or(Msg::Tick);
        let previous_view = (app.selection, app.inspect, app.graph, app.focus());
        match msg {
            Msg::Mouse(mouse) => {
                if app.inspect || app.graph {
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            app.selection = app.selection.saturating_add(1).min(
                                view::navigation_rows(&app.cache.summary, &app.focus(), app.graph)
                                    .len()
                                    .saturating_sub(1),
                            )
                        }
                        MouseEventKind::ScrollUp => app.selection = app.selection.saturating_sub(1),
                        MouseEventKind::Down(event::MouseButton::Left) => {
                            let size = terminal.size()?;
                            if let Some(index) = view::selection_at(
                                size.width,
                                size.height,
                                app.offer,
                                app.selection,
                                mouse.column,
                                mouse.row,
                            ) {
                                app.selection = index.min(
                                    view::navigation_rows(
                                        &app.cache.summary,
                                        &app.focus(),
                                        app.graph,
                                    )
                                    .len()
                                    .saturating_sub(1),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            Msg::Grew => {
                let revision = app.tr.revision;
                if let Err(e) = app.tr.read_new() {
                    app.error = Some(e.to_string());
                }
                if revision != app.tr.revision {
                    app.generation = app.generation.wrapping_add(1);
                    app.analyzing = false;
                    app.cache = Cache {
                        summary: summary::heuristic(&app.tr),
                        ..Cache::default()
                    };
                }
                if app.cache.updated_at == 0 {
                    app.cache.summary = summary::heuristic(&app.tr);
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
                KeyCode::Esc if app.inspect || app.graph => {
                    app.inspect = false;
                    app.graph = false;
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Char('r') => {
                    app.last_summary_started = None;
                    app.dirty = true;
                    app.last_growth = Some(Instant::now() - Duration::from_secs(10));
                    app.cache.turns_done = app
                        .cache
                        .turns_done
                        .min(app.tr.turns.len().saturating_sub(1));
                }
                KeyCode::Char('e') | KeyCode::Enter => {
                    app.inspect = !app.inspect;
                    app.graph = false;
                }
                KeyCode::Char('g') => {
                    app.graph = !app.graph;
                    app.inspect = false;
                    app.selection = 0;
                }
                KeyCode::Down => {
                    app.inspect = !app.graph;
                    app.selection = app.selection.saturating_add(1).min(
                        view::navigation_rows(&app.cache.summary, &app.focus(), app.graph)
                            .len()
                            .saturating_sub(1),
                    );
                }
                KeyCode::Up => {
                    app.inspect = !app.graph;
                    app.selection = app.selection.saturating_sub(1);
                }
                KeyCode::Char('j') => app.scroll = app.scroll.saturating_add(1),
                KeyCode::Char('k') => app.scroll = app.scroll.saturating_sub(1),
                KeyCode::Char('v') => {
                    app.inspect = false;
                    app.graph = false;
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
            Msg::Summary(result, turns_done, generation, snapshot) => {
                let updates = result
                    .as_ref()
                    .as_ref()
                    .map(|s| s.todo_updates.clone())
                    .unwrap_or_default();
                if app.finish_summary(*result, turns_done, generation) {
                    if !updates.is_empty() {
                        if let Err(e) = todos::path(&app.session_id).and_then(|path| {
                            todos::edit(&path, |store| {
                                store.apply(&snapshot, &updates, &app.tr, turns_done);
                                Ok(())
                            })
                        }) {
                            app.error = Some(e.to_string());
                        }
                    }
                    app.cache.summary.todo_updates.clear();
                    if let Err(e) = summary::save_cache(&app.session_id, &app.cache) {
                        app.error = Some(e.to_string());
                    }
                }
            }
        }
        app.selection = app.selection.min(
            view::navigation_rows(&app.cache.summary, &app.focus(), app.graph)
                .len()
                .saturating_sub(1),
        );
        if previous_view != (app.selection, app.inspect, app.graph, app.focus()) {
            app.scroll = 0;
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
    let Ok(path) = transcript::find_transcript(&sid).or_else(|e| {
        info.cwd
            .as_deref()
            .map(|cwd| transcript::expected_path(cwd, &sid))
            .unwrap_or(Err(e))
    }) else {
        return;
    };
    let mut tr = Transcript::open(&path);
    if tr.read_new().is_err() {
        return;
    }
    app.cache = summary::load_cache(&sid)
        .filter(|c| c.turns_done <= tr.turns.len() && c.fingerprint == tr.fingerprint(c.turns_done))
        .unwrap_or_else(|| Cache {
            version: summary::CACHE_VERSION,
            summary: summary::heuristic(&tr),
            turns_done: 0,
            fingerprint: 0,
            updated_at: 0,
            source: "heuristic".into(),
        });
    if let Ok(mut guard) = app.watched_path.lock() {
        *guard = path;
    }
    app.tr = tr;
    app.session_id = sid;
    app.generation = app.generation.wrapping_add(1);
    app.analyzing = false;
    app.last_summary_started = None;
    app.pinned = false;
    app.focus_override = None;
    app.dirty = true;
    app.scroll = 0;
    app.selection = 0;
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
    if app
        .last_summary_started
        .is_some_and(|t| t.elapsed() < Duration::from_secs(app.refresh_seconds))
    {
        return;
    }
    let settled = app
        .last_growth
        .map(|t| t.elapsed() >= Duration::from_secs(2))
        .unwrap_or(true);
    if !settled {
        return;
    }
    let (text, turns_done) = summary::pending_chunk(&app.tr, app.cache.turns_done);
    let prev = app.cache.summary.clone();
    let title = app
        .tr
        .free
        .custom_title
        .clone()
        .or_else(|| app.tr.free.title.clone());
    let generation = app.generation;
    let snapshot = match todos::path(&app.session_id).and_then(|p| todos::load(&p)) {
        Ok(store) => store.items,
        Err(e) => {
            app.error = Some(e.to_string());
            app.dirty = false;
            return;
        }
    };
    let tx = app.tx.clone();
    app.analyzing = true;
    app.last_summary_started = Some(Instant::now());
    app.dirty = false;
    thread::spawn(move || {
        let result = summary::summarize(&prev, &text, title.as_deref(), &snapshot);
        let _ = tx.send(Msg::Summary(
            Box::new(result),
            turns_done,
            generation,
            snapshot,
        ));
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
        inspect: app.inspect,
        graph: app.graph,
        selection: app.selection,
        transcript: &app.tr,
    };
    terminal.draw(|f| view::draw(f, &state))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_switch_rejects_old_results_without_releasing_new_worker() {
        let (tx, _) = mpsc::channel();
        let path = std::path::PathBuf::from("unused.jsonl");
        let mut app = App {
            session_id: "new-session".into(),
            client: None,
            pane: None,
            watched_path: Arc::new(Mutex::new(path.clone())),
            tr: Transcript::open(&path),
            cache: Cache::default(),
            agent_status: None,
            analyzing: true,
            generation: 2,
            refresh_seconds: 30,
            last_summary_started: None,
            dirty: false,
            last_growth: None,
            error: None,
            scroll: 0,
            no_model: true,
            offer: false,
            focus_override: None,
            pinned: false,
            rail: false,
            inspect: false,
            graph: false,
            selection: 0,
            tx,
        };
        let old = Summary {
            topline: "old session".into(),
            ..Summary::default()
        };
        assert!(!app.finish_summary(Ok(old), 900, 1));
        assert_eq!(app.cache.turns_done, 0);
        assert!(app.cache.summary.topline.is_empty());
        assert!(app.analyzing);
        let new = Summary {
            topline: "new session".into(),
            ..Summary::default()
        };
        assert!(app.finish_summary(Ok(new), 3, 2));
        assert_eq!(app.cache.turns_done, 3);
        assert_eq!(app.cache.summary.topline, "new session");
        assert!(!app.analyzing);
    }
}
