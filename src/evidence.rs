//! Provenance, item navigation, and portable session graph export.
use crate::summary::Summary;
use crate::transcript::{clip, Transcript, Turn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub source_turns: Vec<usize>,
    #[serde(default)]
    pub from: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: String,
    pub text: String,
    pub kind: String,
    pub branch: String,
    pub status: String,
    pub from: Option<String>,
    pub source_turns: Vec<usize>,
}

pub fn nodes(summary: &Summary) -> Vec<Node> {
    let mut nodes = Vec::new();
    for p in &summary.plan {
        nodes.push(Node {
            id: p.evidence.id.clone(),
            text: p.text.clone(),
            kind: "plan".into(),
            branch: p.branch.clone(),
            status: p.status.clone(),
            from: p.evidence.from.clone(),
            source_turns: p.evidence.source_turns.clone(),
        });
    }
    for (kind, items) in [
        ("question", &summary.open_questions),
        ("decision", &summary.decisions),
        ("blocker", &summary.blockers),
    ] {
        for item in items {
            nodes.push(Node {
                id: item.evidence.id.clone(),
                text: item.text.clone(),
                kind: kind.into(),
                branch: item.branch.clone(),
                status: String::new(),
                from: item.evidence.from.clone(),
                source_turns: item.evidence.source_turns.clone(),
            });
        }
    }
    nodes
}

pub fn normalize(summary: &mut Summary, turns: usize) {
    let mut all: Vec<&mut Evidence> = summary
        .plan
        .iter_mut()
        .map(|p| &mut p.evidence)
        .chain(
            summary
                .open_questions
                .iter_mut()
                .chain(&mut summary.decisions)
                .chain(&mut summary.blockers)
                .map(|i| &mut i.evidence),
        )
        .collect();
    let mut ids = HashSet::new();
    for (index, e) in all.iter_mut().enumerate() {
        if e.id.is_empty() || !ids.insert(e.id.clone()) {
            let mut id = format!("item-{index}");
            while ids.contains(&id) {
                id.push('x');
            }
            ids.insert(id.clone());
            e.id = id;
        }
        e.source_turns.retain(|n| *n < turns);
        e.source_turns.sort_unstable();
        e.source_turns.dedup();
        e.source_turns.truncate(8);
    }
    let parents: HashMap<String, Option<String>> =
        all.iter().map(|e| (e.id.clone(), e.from.clone())).collect();
    for e in all {
        let mut seen = HashSet::from([e.id.clone()]);
        let mut next = e.from.as_ref();
        while let Some(id) = next {
            if !ids.contains(id) || !seen.insert(id.clone()) {
                e.from = None;
                break;
            }
            next = parents.get(id).and_then(Option::as_ref);
        }
    }
}

pub fn excerpt(tr: &Transcript, sources: &[usize]) -> String {
    if sources.is_empty() {
        return "No source turns supplied. Verify this item in the conversation.".into();
    }
    sources
        .iter()
        .filter_map(|n| {
            tr.turns.get(*n).map(|t| {
                let (role, text) = match t {
                    Turn::User(s) => ("USER", s),
                    Turn::Assistant(s) => ("ASSISTANT", s),
                    Turn::Tool(s) => ("TOOL", s),
                };
                format!("[t{n}] {role}\n{}", clip(text, 4000))
            })
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A forest of explicit item relationships; branch lanes remain available in the rail.
pub fn graph_rows(nodes: &[Node]) -> Vec<(usize, usize)> {
    fn visit(
        nodes: &[Node],
        idx: usize,
        depth: usize,
        seen: &mut HashSet<usize>,
        rows: &mut Vec<(usize, usize)>,
    ) {
        if !seen.insert(idx) {
            return;
        }
        rows.push((idx, depth));
        for (child, node) in nodes.iter().enumerate() {
            if node.from.as_deref() == Some(&nodes[idx].id) {
                visit(nodes, child, depth + 1, seen, rows);
            }
        }
    }
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    for (idx, n) in nodes.iter().enumerate().filter(|(_, n)| n.from.is_none()) {
        let _ = n;
        visit(nodes, idx, 0, &mut seen, &mut rows);
    }
    for idx in 0..nodes.len() {
        visit(nodes, idx, 0, &mut seen, &mut rows);
    }
    rows
}

pub fn html(summary: &Summary, tr: &Transcript) -> anyhow::Result<String> {
    let items = nodes(summary);
    let data = serde_json::json!({"title":summary.topline, "nodes":items.iter().map(|n| {
        serde_json::json!({"item":n,"evidence":excerpt(tr, &n.source_turns)})
    }).collect::<Vec<_>>(), "rows":graph_rows(&items)});
    let json = serde_json::to_string(&data)?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    Ok(include_str!("graph.html").replace("__GLANCE_DATA__", &json))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_sources_and_relationship_cycles_are_removed() {
        let mut s: Summary = serde_json::from_value(serde_json::json!({"plan":[
            {"text":"a","status":"done","id":"a","from":"b","source_turns":[0,0,99]},
            {"text":"b","status":"pending","id":"b","from":"a"}
        ]}))
        .unwrap();
        normalize(&mut s, 2);
        assert_eq!(s.plan[0].evidence.source_turns, vec![0]);
        assert!(s.plan.iter().all(|p| p.evidence.from.is_none()));
    }

    #[test]
    fn graph_export_escapes_script_content_and_preserves_relationships() {
        let mut s: Summary = serde_json::from_value(
            serde_json::json!({"topline":"</script><img onerror=alert(1)>","plan":[
            {"text":"parent","status":"done","id":"p","source_turns":[0]}
        ],"decisions":[{"text":"child","id":"d","from":"p","source_turns":[0]}]}),
        )
        .unwrap();
        normalize(&mut s, 1);
        let mut tr = Transcript::open(std::path::Path::new("unused"));
        tr.turns.push(Turn::User("Evidence".into()));
        assert_eq!(graph_rows(&nodes(&s)), vec![(0, 0), (1, 1)]);
        let html = html(&s, &tr).unwrap();
        assert!(!html.contains("</script><img"));
        assert!(html.contains("\\u003c/script\\u003e"));
        assert!(html.contains("Evidence"));
    }
}
