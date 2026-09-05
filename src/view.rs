//! Fixed-layout rendering of the panel (and the rail view) with ratatui.

use crate::summary::{Item, PlanItem, Summary};
use crate::transcript::{clip, Free};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// What the panel is oriented on.
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    All,
    Trunk,
    Branch(String),
}

pub struct ViewState<'a> {
    pub free: &'a Free,
    pub summary: &'a Summary,
    pub agent_status: Option<&'a str>,
    pub source: &'a str,
    pub updated_at: u64,
    pub analyzing: bool,
    pub error: Option<&'a str>,
    pub scroll: u16,
    pub waiting: bool,
    pub offer: bool,
    pub focus: Focus,
    pub pinned: bool,
    pub rail: bool,
    pub inspect: bool,
    pub graph: bool,
    pub selection: usize,
    pub transcript: &'a crate::transcript::Transcript,
    pub todos: &'a [crate::todos::Todo],
    pub todo_mode: bool,
    pub todo_input: Option<&'a str>,
}

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;

trait Branched {
    fn branch(&self) -> &str;
    fn text(&self) -> &str;
}
impl Branched for PlanItem {
    fn branch(&self) -> &str {
        &self.branch
    }
    fn text(&self) -> &str {
        &self.text
    }
}
impl Branched for Item {
    fn branch(&self) -> &str {
        &self.branch
    }
    fn text(&self) -> &str {
        &self.text
    }
}

/// Items to show for the current focus: (item, dimmed, branch-tag prefix).
fn visible<'a, T: Branched>(items: &'a [T], s: &ViewState) -> Vec<(&'a T, bool, String)> {
    let multi = s.summary.is_multi();
    items
        .iter()
        .filter_map(|it| match &s.focus {
            Focus::All => {
                let tag = if multi && it.branch() != "trunk" {
                    format!("[{}] ", s.summary.branch_name(it.branch()))
                } else {
                    String::new()
                };
                Some((it, false, tag))
            }
            Focus::Trunk => (it.branch() == "trunk").then(|| (it, false, String::new())),
            Focus::Branch(id) => {
                if it.branch() == id {
                    Some((it, false, String::new()))
                } else if it.branch() == "trunk" {
                    Some((it, true, String::new()))
                } else {
                    None
                }
            }
        })
        .collect()
}

pub fn draw(f: &mut Frame, s: &ViewState) {
    let area = f.area();
    let offer_h = if s.offer { 4 } else { 0 };
    let chunks = Layout::vertical([
        Constraint::Length(offer_h),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    if s.offer {
        let banner = Paragraph::new(Text::from(vec![
            Line::from(vec![Span::styled(
                " Open glance automatically for every Claude Code session? ",
                Style::default().bold(),
            )]),
            Line::from(vec![
                Span::styled(" y ", Style::default().fg(Color::Black).bg(ACCENT).bold()),
                Span::raw(" yes, add the SessionStart hook   "),
                Span::styled(" n ", Style::default().fg(Color::Black).bg(DIM).bold()),
                Span::raw(" no, don't ask again"),
            ]),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ACCENT)),
        )
        .wrap(Wrap { trim: false });
        f.render_widget(banner, chunks[0]);
    }

    if s.inspect || s.graph || s.todo_mode {
        draw_navigation(f, s, chunks[1]);
        f.render_widget(footer(s, area.width), chunks[2]);
        return;
    }

    let title = s
        .free
        .custom_title
        .as_deref()
        .or(s.free.title.as_deref())
        .unwrap_or("Claude Code session");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(clip(title, 60), Style::default().bold()),
            Span::raw(" "),
        ]));

    let inner_w = chunks[1].width.saturating_sub(2) as usize;
    let body = if s.rail {
        Paragraph::new(Text::from(rail_lines(s, inner_w)))
            .block(block)
            .scroll((s.scroll, 0))
    } else {
        Paragraph::new(Text::from(panel_lines(s, inner_w)))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((s.scroll, 0))
    };
    f.render_widget(body, chunks[1]);
    f.render_widget(footer(s, area.width), chunks[2]);
}

pub fn navigation_rows(
    summary: &Summary,
    focus: &Focus,
    graph: bool,
) -> Vec<(crate::evidence::Node, usize)> {
    let nodes = crate::evidence::nodes(summary);
    let order = if graph {
        crate::evidence::graph_rows(&nodes)
    } else {
        (0..nodes.len()).map(|i| (i, 0)).collect()
    };
    order
        .into_iter()
        .filter_map(|(i, depth)| {
            let n = &nodes[i];
            let visible = match focus {
                Focus::All => true,
                Focus::Trunk => n.branch == "trunk",
                Focus::Branch(id) => n.branch == "trunk" || n.branch == *id,
            };
            visible.then(|| (n.clone(), depth))
        })
        .collect()
}

fn navigation_areas(area: Rect) -> [Rect; 3] {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length((area.height / 2).clamp(4, 12)),
    ])
    .split(area);
    [chunks[0], chunks[1], chunks[2]]
}

pub fn selection_at(
    width: u16,
    height: u16,
    offer: bool,
    selected: usize,
    x: u16,
    y: u16,
) -> Option<usize> {
    let chunks = Layout::vertical([
        Constraint::Length(if offer { 4 } else { 0 }),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(Rect::new(0, 0, width, height));
    let list = navigation_areas(chunks[1])[1];
    let inner = Block::default().borders(Borders::ALL).inner(list);
    if !inner.contains((x, y).into()) {
        return None;
    }
    let offset = selected.saturating_sub(inner.height.saturating_sub(1) as usize);
    Some(offset + (y - inner.y) as usize)
}

fn draw_navigation(f: &mut Frame, s: &ViewState, area: Rect) {
    let [header, list_area, drawer] = navigation_areas(area);
    f.render_widget(
        Paragraph::new(s.summary.topline.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(if s.todo_mode {
                    " MY TODOS · a add · x toggle · d delete "
                } else if s.graph {
                    " SESSION GRAPH "
                } else {
                    " EXPLORE ITEMS "
                }),
        ),
        header,
    );
    let rows = if s.todo_mode {
        s.todos
            .iter()
            .map(|t| {
                (
                    crate::evidence::Node {
                        id: t.id.clone(),
                        text: format!("{} {}", plan_glyph(t.status.label()).0, t.text),
                        kind: "todo".into(),
                        branch: "trunk".into(),
                        status: t.status.label().into(),
                        from: None,
                        source_turns: t.source_turns.clone(),
                    },
                    0,
                )
            })
            .collect()
    } else {
        navigation_rows(s.summary, &s.focus, s.graph)
    };
    let selected = s.selection.min(rows.len().saturating_sub(1));
    let entries: Vec<ListItem> = rows
        .iter()
        .map(|(n, depth)| {
            let color = match n.kind.as_str() {
                "question" => Color::Magenta,
                "blocker" => Color::Red,
                "decision" => ACCENT,
                _ => Color::Yellow,
            };
            let prefix = if s.graph && *depth > 0 {
                format!("{}└─ ", "  ".repeat((*depth).min(8)))
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{prefix}[{}] ", n.id), Style::default().fg(DIM)),
                Span::styled(n.text.clone(), Style::default().fg(color)),
            ]))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ↑ ↓ / click select · j k scroll evidence · Esc back ");
    let capacity = block.inner(list_area).height;
    let mut state = ListState::default().with_selected((!rows.is_empty()).then_some(selected));
    *state.offset_mut() = selected.saturating_sub(capacity.saturating_sub(1) as usize);
    f.render_stateful_widget(
        List::new(entries)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .highlight_symbol("› "),
        list_area,
        &mut state,
    );
    let text = rows
        .get(selected)
        .map(|(n, _)| {
            let source = crate::evidence::excerpt(s.transcript, &n.source_turns);
            let parent = n
                .from
                .as_deref()
                .map(|p| format!("From item: {p}\n\n"))
                .unwrap_or_default();
            let note = if s.todo_mode {
                s.todos
                    .get(selected)
                    .map(|t| {
                        format!(
                            "{} · set by {}\n{}\n\n",
                            t.status.label().replace('_', " "),
                            t.status_by,
                            t.note
                        )
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };
            format!("{note}{parent}{source}")
        })
        .unwrap_or_else(|| {
            if s.todo_mode {
                "Press a to add your first reminder.".into()
            } else {
                "No summary items yet.".into()
            }
        });
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((s.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" TRANSCRIPT EVIDENCE "),
            ),
        drawer,
    );
}

fn panel_lines(s: &ViewState, inner_w: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(header_line(s));
    lines.push(Line::raw(""));

    section(&mut lines, "WHAT WE ARE WORKING ON");
    push_text_or_dash(&mut lines, &s.summary.topline, Style::default());
    lines.push(Line::raw(""));

    section(&mut lines, "NOW");
    if s.summary.now.is_empty() {
        lines.push(Line::styled("–", Style::default().fg(DIM)));
    } else {
        lines.push(Line::from(vec![
            Span::styled("▶ ", Style::default().fg(Color::Yellow)),
            Span::raw(s.summary.now.clone()),
        ]));
    }
    lines.push(Line::raw(""));

    if s.summary.is_multi() {
        let hdr = match &s.focus {
            Focus::All => "BRANCHES · showing all".to_string(),
            Focus::Trunk => "BRANCHES · trunk".to_string(),
            Focus::Branch(id) => format!("BRANCHES · {}", s.summary.branch_name(id)),
        };
        section(&mut lines, &hdr);
        lines.extend(branch_chips(s, inner_w));
        // The focused branch's one-line summary.
        if let Focus::Branch(id) = &s.focus {
            if let Some(b) = s.summary.branches.iter().find(|b| &b.id == id) {
                if !b.summary.is_empty() {
                    lines.push(Line::styled(
                        b.summary.clone(),
                        Style::default().fg(Color::Gray),
                    ));
                }
            }
        }
        lines.push(Line::raw(""));
    }

    let plan = visible(&s.summary.plan, s);
    let done = plan
        .iter()
        .filter(|(p, dim, _)| !dim && p.status == "done")
        .count();
    let total = plan.iter().filter(|(_, dim, _)| !dim).count();
    section(
        &mut lines,
        &if total == 0 {
            "PLAN".to_string()
        } else {
            format!("PLAN  {done}/{total}")
        },
    );
    if plan.is_empty() {
        lines.push(Line::styled("–", Style::default().fg(DIM)));
    }
    for (item, dim, tag) in plan {
        let (mark, style) = plan_glyph(&item.status);
        let text_style = if dim || item.status == "done" {
            Style::default().fg(DIM)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(mark, if dim { Style::default().fg(DIM) } else { style }),
            Span::styled(tag, Style::default().fg(DIM)),
            Span::styled(item.text.clone(), text_style),
        ]));
    }
    lines.push(Line::raw(""));

    section(&mut lines, "MY TODOS · a add · t manage");
    if s.todos.is_empty() {
        lines.push(Line::styled("–", Style::default().fg(DIM)));
    }
    for item in s.todos {
        let (mark, style) = plan_glyph(item.status.label());
        lines.push(Line::from(vec![
            Span::styled(mark, style),
            Span::raw(item.text.clone()),
        ]));
    }
    lines.push(Line::raw(""));

    section(&mut lines, "OPEN QUESTIONS");
    list(
        &mut lines,
        visible(&s.summary.open_questions, s),
        "? ",
        Color::Magenta,
    );
    lines.push(Line::raw(""));

    section(&mut lines, "DECISIONS");
    list(&mut lines, visible(&s.summary.decisions, s), "· ", ACCENT);
    lines.push(Line::raw(""));

    let blockers = visible(&s.summary.blockers, s);
    if !blockers.is_empty() {
        section(&mut lines, "BLOCKED ON");
        list(&mut lines, blockers, "! ", Color::Red);
        lines.push(Line::raw(""));
    }

    section(&mut lines, "LAST FROM CLAUDE");
    let last = s
        .free
        .last_assistant
        .as_deref()
        .map(|t| clip(t, 320))
        .unwrap_or_default();
    push_text_or_dash(&mut lines, &last, Style::default().fg(Color::Gray));
    lines
}

fn plan_glyph(status: &str) -> (&'static str, Style) {
    match status {
        "done" => ("✔ ", Style::default().fg(Color::Green)),
        "in_progress" => ("▶ ", Style::default().fg(Color::Yellow)),
        "blocked" => ("✖ ", Style::default().fg(Color::Red)),
        _ => ("○ ", Style::default().fg(DIM)),
    }
}

fn branch_glyph(status: &str) -> (&'static str, Color) {
    match status {
        "done" => ("✔", Color::Green),
        "parked" => ("◌", DIM),
        _ => ("●", Color::Yellow),
    }
}

fn branch_chips(s: &ViewState, width: usize) -> Vec<Line<'static>> {
    let sel = Style::default().fg(Color::Black).bg(ACCENT).bold();
    let mut chips: Vec<Vec<Span>> = Vec::new();
    let trunk_sel = s.focus == Focus::Trunk;
    chips.push(vec![Span::styled(
        " ⌂ trunk ",
        if trunk_sel {
            sel
        } else {
            Style::default().fg(Color::Gray)
        },
    )]);
    let mut done_shown = 0;
    let mut done_hidden = 0;
    for b in &s.summary.branches {
        let (g, c) = branch_glyph(&b.status);
        let selected = matches!(&s.focus, Focus::Branch(id) if id == &b.id);
        if b.status == "done" && !selected {
            if done_shown >= 2 {
                done_hidden += 1;
                continue;
            }
            done_shown += 1;
        }
        if selected {
            chips.push(vec![Span::styled(format!(" {g} {} ", b.name), sel)]);
        } else {
            chips.push(vec![
                Span::styled(format!("{g} "), Style::default().fg(c)),
                Span::styled(
                    b.name.clone(),
                    Style::default().fg(if b.status == "done" {
                        DIM
                    } else {
                        Color::Reset
                    }),
                ),
            ]);
        }
    }
    if done_hidden > 0 {
        chips.push(vec![Span::styled(
            format!("+{done_hidden} done"),
            Style::default().fg(DIM),
        )]);
    }
    pack_chips(chips, width, " ")
}

/// Lay chips (each a group of spans that must stay together) into as many lines as needed.
fn pack_chips(chips: Vec<Vec<Span<'static>>>, width: usize, gap: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut cur: Vec<Span> = Vec::new();
    let mut cur_w = 0usize;
    for chip in chips {
        let w: usize = chip.iter().map(Span::width).sum();
        let sep = if cur.is_empty() {
            0
        } else {
            gap.chars().count()
        };
        if !cur.is_empty() && cur_w + sep + w > width {
            lines.push(Line::from(std::mem::take(&mut cur)));
            cur_w = 0;
        }
        if !cur.is_empty() {
            cur.push(Span::raw(gap.to_string()));
            cur_w += sep;
        }
        cur_w += w;
        cur.extend(chip);
    }
    if !cur.is_empty() {
        lines.push(Line::from(cur));
    }
    lines
}

fn header_line(s: &ViewState) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    if let Some(wt) = &s.free.worktree {
        spans.push(Span::styled(format!("⌥ {wt}"), Style::default().fg(ACCENT)));
    } else if let Some(b) = &s.free.branch {
        spans.push(Span::styled(format!("⌥ {b}"), Style::default().fg(ACCENT)));
    }
    if let Some(pr) = s.free.pr_number {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("PR #{pr}"),
            Style::default().fg(ACCENT),
        ));
    }
    let (dot, color, label) = match s.agent_status {
        Some("working") => ("●", Color::Yellow, "working"),
        Some("blocked") => ("●", Color::Red, "needs you"),
        Some("done") => ("●", Color::Blue, "done"),
        Some("idle") => ("●", Color::Green, "idle"),
        Some(other) => ("●", DIM, other),
        None => ("○", DIM, "no herdr"),
    };
    spans.push(Span::raw("  "));
    spans.push(Span::styled(dot, Style::default().fg(color)));
    spans.push(Span::styled(
        format!(" {label}"),
        Style::default().fg(color),
    ));
    Line::from(spans)
}

fn section(lines: &mut Vec<Line<'static>>, title: &str) {
    lines.push(Line::styled(
        title.to_string(),
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ));
}

fn list<T: Branched>(
    lines: &mut Vec<Line<'static>>,
    items: Vec<(&T, bool, String)>,
    mark: &str,
    color: Color,
) {
    if items.is_empty() {
        lines.push(Line::styled("–", Style::default().fg(DIM)));
    }
    for (it, dim, tag) in items {
        let style = if dim {
            Style::default().fg(DIM)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(
                mark.to_string(),
                if dim {
                    Style::default().fg(DIM)
                } else {
                    Style::default().fg(color)
                },
            ),
            Span::styled(tag, Style::default().fg(DIM)),
            Span::styled(it.text().to_string(), style),
        ]));
    }
}

fn push_text_or_dash(lines: &mut Vec<Line<'static>>, text: &str, style: Style) {
    if text.trim().is_empty() {
        lines.push(Line::styled("–", Style::default().fg(DIM)));
    } else {
        lines.push(Line::styled(text.to_string(), style));
    }
}

// ---------------------------------------------------------------------------------------------
// Rail view: trunk plus one lane per branch, time flowing down, one row per item.

struct Node {
    turn: usize,
    branch: String,
    glyph: &'static str,
    color: Color,
    text: String,
}

fn rail_lines(s: &ViewState, inner_w: usize) -> Vec<Line<'static>> {
    let sm = s.summary;
    let mut nodes: Vec<Node> = Vec::new();
    for p in &sm.plan {
        let (g, st) = plan_glyph(&p.status);
        nodes.push(Node {
            turn: p.turn,
            branch: p.branch.clone(),
            glyph: g.trim_end(),
            color: st.fg.unwrap_or(DIM),
            text: p.text.clone(),
        });
    }
    for q in &sm.open_questions {
        nodes.push(Node {
            turn: q.turn,
            branch: q.branch.clone(),
            glyph: "?",
            color: Color::Magenta,
            text: q.text.clone(),
        });
    }
    for d in &sm.decisions {
        nodes.push(Node {
            turn: d.turn,
            branch: d.branch.clone(),
            glyph: "·",
            color: ACCENT,
            text: d.text.clone(),
        });
    }
    for b in &sm.blockers {
        nodes.push(Node {
            turn: b.turn,
            branch: b.branch.clone(),
            glyph: "!",
            color: Color::Red,
            text: b.text.clone(),
        });
    }
    nodes.sort_by_key(|n| n.turn);

    // Lane order: trunk, then branches by first appearance (row order, so lanes open at their first row).
    let first_turn = |id: &str| nodes.iter().position(|n| n.branch == id);
    let last_turn = |id: &str| nodes.iter().rposition(|n| n.branch == id);
    let mut branches: Vec<&crate::summary::Branch> = sm.branches.iter().collect();
    branches.sort_by_key(|b| first_turn(&b.id).unwrap_or(usize::MAX));
    let fit = 1 + inner_w.saturating_sub(34) / 2;
    let (shown, folded): (Vec<_>, Vec<_>) = branches
        .iter()
        .enumerate()
        .partition(|(i, _)| *i < fit.saturating_sub(1));
    let shown: Vec<&crate::summary::Branch> = shown.into_iter().map(|(_, b)| *b).collect();
    let folded: Vec<&crate::summary::Branch> = folded.into_iter().map(|(_, b)| *b).collect();
    let lanes: Vec<String> = std::iter::once("trunk".to_string())
        .chain(shown.iter().map(|b| b.id.clone()))
        .collect();
    let lane_w = lanes.len() * 2 + if folded.is_empty() { 0 } else { 2 };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(header_line(s));
    // Legend, packed into as many lines as it needs.
    let mut chips: Vec<Vec<Span>> = vec![vec![Span::styled(
        "⌂ trunk",
        Style::default().fg(Color::Gray),
    )]];
    for b in &shown {
        let (g, c) = branch_glyph(&b.status);
        chips.push(vec![
            Span::styled(format!("{g} "), Style::default().fg(c)),
            Span::raw(clip(&b.name, 18)),
        ]);
    }
    if !folded.is_empty() {
        chips.push(vec![Span::styled(
            format!("+{} more", folded.len()),
            Style::default().fg(DIM),
        )]);
    }
    lines.extend(pack_chips(chips, inner_w, "  "));
    lines.push(Line::raw(""));
    if nodes.is_empty() {
        lines.push(Line::styled(
            "nothing to draw yet",
            Style::default().fg(DIM),
        ));
        return lines;
    }

    let alive = |id: &str, row: usize| -> bool {
        let Some(first) = first_turn(id) else {
            return false;
        };
        if row < first {
            return false;
        }
        match sm.branches.iter().find(|b| b.id == id) {
            Some(b) if b.status == "done" => row <= last_turn(id).unwrap_or(row),
            _ => true,
        }
    };
    let text_w = inner_w.saturating_sub(lane_w + 1);
    for (row, n) in nodes.iter().enumerate() {
        let is_folded = folded.iter().any(|b| b.id == n.branch);
        let target = lanes.iter().position(|l| l == &n.branch);
        let is_done = sm
            .branches
            .iter()
            .any(|b| b.id == n.branch && b.status == "done");
        let connector = target
            .map(|t| {
                t > 0
                    && (first_turn(&n.branch) == Some(row)
                        || (is_done && last_turn(&n.branch) == Some(row)))
            })
            .unwrap_or(false);
        let mut spans: Vec<Span> = Vec::new();
        for (li, lane) in lanes.iter().enumerate() {
            let on_path = connector && target.map(|t| li < t).unwrap_or(false);
            if Some(li) == target {
                spans.push(Span::styled(
                    n.glyph.to_string(),
                    Style::default().fg(n.color),
                ));
                spans.push(Span::raw(" "));
            } else if li == 0 {
                let g = if on_path { "├─" } else { "│ " };
                spans.push(Span::styled(
                    g.to_string(),
                    Style::default().fg(Color::Gray),
                ));
            } else if alive(lane, row) {
                let g = if on_path { "┼─" } else { "│ " };
                spans.push(Span::styled(g.to_string(), Style::default().fg(DIM)));
            } else {
                spans.push(Span::styled(
                    if on_path { "──" } else { "  " }.to_string(),
                    Style::default().fg(DIM),
                ));
            }
        }
        let mut text = n.text.clone();
        if is_folded {
            spans.push(Span::styled(
                n.glyph.to_string(),
                Style::default().fg(n.color),
            ));
            spans.push(Span::raw(" "));
            text = format!("[{}] {}", sm.branch_name(&n.branch), text);
        }
        spans.push(Span::styled(clip(&text, text_w), Style::default()));
        lines.push(Line::from(spans));
    }
    lines
}

fn footer(s: &ViewState, width: u16) -> Paragraph<'static> {
    if let Some(input) = s.todo_input {
        let line = Line::raw(format!(" Add todo (Enter save / Esc cancel): {input}▏"));
        let offset = line
            .width()
            .saturating_sub(width as usize)
            .min(u16::MAX as usize) as u16;
        return Paragraph::new(line)
            .scroll((0, offset))
            .style(Style::default().fg(Color::Black).bg(ACCENT));
    }
    let age = crate::summary::now_secs().saturating_sub(s.updated_at);
    let age_txt = if s.updated_at == 0 {
        "never".to_string()
    } else if age < 60 {
        format!("{age}s ago")
    } else if age < 3600 {
        format!("{}m ago", age / 60)
    } else {
        format!("{}h ago", age / 3600)
    };
    let mut spans = vec![Span::styled(
        format!(" {} · updated {age_txt}", s.source),
        Style::default().fg(DIM),
    )];
    if let Some(usage) = &s.summary.usage {
        let cost = usage
            .estimated_usd
            .map(|v| format!(" · ~${v:.3}"))
            .unwrap_or_default();
        spans.push(Span::styled(
            format!(" · {} calls{cost}", usage.calls),
            Style::default().fg(DIM),
        ));
    }
    if s.waiting {
        spans.push(Span::styled(
            "  waiting for the first prompt",
            Style::default().fg(Color::Yellow),
        ));
    } else if s.analyzing {
        spans.push(Span::styled(
            "  ⟳ analyzing…",
            Style::default().fg(Color::Yellow),
        ));
    }
    if let Some(e) = s.error {
        spans.push(Span::styled(
            format!("  ⚠ {}", clip(e, 60)),
            Style::default().fg(Color::Red),
        ));
    }
    if s.pinned {
        spans.push(Span::styled("  pinned", Style::default().fg(ACCENT)));
    }
    let keys = if s.inspect || s.graph {
        "  ↑↓ select · e evidence · g graph · Esc back"
    } else if s.rail {
        "  v panel · j/k scroll · q quit"
    } else if s.summary.is_multi() {
        "  [ ] focus · 0 all · p follow · v rail · q quit"
    } else {
        "  e evidence · g graph · v rail · r refresh · q quit"
    };
    spans.push(Span::styled(keys.to_string(), Style::default().fg(DIM)));
    Paragraph::new(Line::from(spans))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summary::Branch;

    #[test]
    fn evidence_drawer_renders_sources_and_mouse_uses_the_list_geometry() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut tr = crate::transcript::Transcript::open(std::path::Path::new("unused"));
        tr.turns.push(crate::transcript::Turn::User(
            "The retry test passed.".into(),
        ));
        let mut summary: Summary = serde_json::from_value(serde_json::json!({"topline":"Verify retries", "plan":[{"id":"p1","text":"Run the test","status":"done","source_turns":[0]}]})).unwrap();
        summary.normalize();
        let mut state = ViewState {
            free: &tr.free,
            summary: &summary,
            agent_status: None,
            source: "fixture",
            updated_at: 0,
            analyzing: false,
            error: None,
            scroll: 0,
            waiting: false,
            offer: false,
            focus: Focus::All,
            pinned: false,
            rail: false,
            inspect: true,
            graph: false,
            selection: 0,
            transcript: &tr,
            todos: &[],
            todo_mode: false,
            todo_input: None,
        };
        for (width, height) in [(64, 28), (120, 40), (20, 8), (1, 1)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|f| draw(f, &state)).unwrap();
            if width == 64 {
                let text = terminal
                    .backend()
                    .buffer()
                    .content
                    .iter()
                    .map(|c| c.symbol())
                    .collect::<String>();
                assert!(text.contains("The retry test passed."));
                assert_eq!(selection_at(width, height, false, 0, 2, 4), Some(0));
                assert_eq!(selection_at(width, height, false, 0, 2, 20), None);
            }
        }
        let mut store = crate::todos::Store::default();
        store
            .add("Ask about café rollout", 0, tr.fingerprint(0))
            .unwrap();
        state.todos = &store.items;
        state.todo_mode = true;
        state.todo_input = Some("A long reminder that should keep its ending visible while typing");
        for (width, height) in [(64, 28), (20, 8), (1, 1)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|f| draw(f, &state)).unwrap();
            let text = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|c| c.symbol())
                .collect::<String>();
            if width > 1 {
                assert!(text.contains('▏'), "width {width}: {text}");
            }
            if width == 64 {
                assert!(text.contains("Ask about café rollout"));
            }
        }
    }

    fn chip(text: &str) -> Vec<Span<'static>> {
        vec![Span::raw(text.to_string())]
    }

    #[test]
    fn pack_chips_wraps_whole_chips() {
        let lines = pack_chips(vec![chip("aaaa"), chip("bbbb"), chip("cccc")], 10, " ");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].to_string(), "aaaa bbbb");
        assert_eq!(lines[1].to_string(), "cccc");
    }

    #[test]
    fn pack_chips_never_splits_an_oversized_chip() {
        let lines = pack_chips(vec![chip("toolongforthewidth")], 5, " ");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].to_string(), "toolongforthewidth");
    }

    #[test]
    fn focus_filters_and_dims() {
        let summary = Summary {
            branches: vec![Branch {
                id: "b".into(),
                name: "B".into(),
                status: "active".into(),
                summary: String::new(),
            }],
            plan: vec![
                PlanItem {
                    text: "trunk item".into(),
                    status: "pending".into(),
                    branch: "trunk".into(),
                    turn: 0,
                    ..Default::default()
                },
                PlanItem {
                    text: "branch item".into(),
                    status: "pending".into(),
                    branch: "b".into(),
                    turn: 1,
                    ..Default::default()
                },
            ],
            ..Summary::default()
        };
        let free = Free::default();
        let tr = crate::transcript::Transcript::open(std::path::Path::new("unused"));
        let mk = |focus: Focus| ViewState {
            free: &free,
            summary: &summary,
            agent_status: None,
            source: "x",
            updated_at: 0,
            analyzing: false,
            error: None,
            scroll: 0,
            waiting: false,
            offer: false,
            focus,
            pinned: false,
            rail: false,
            inspect: false,
            graph: false,
            selection: 0,
            transcript: &tr,
            todos: &[],
            todo_mode: false,
            todo_input: None,
        };
        let all = visible(&summary.plan, &mk(Focus::All));
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].2, "[B] ");
        let trunk = visible(&summary.plan, &mk(Focus::Trunk));
        assert_eq!(trunk.len(), 1);
        let b = visible(&summary.plan, &mk(Focus::Branch("b".into())));
        assert_eq!(b.len(), 2);
        assert!(b[0].1, "trunk item is dimmed under a branch focus");
        assert!(!b[1].1);
    }
}
