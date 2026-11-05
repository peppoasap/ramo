//! Parent-derived topology, ported from the released renderer. No date/ref heuristics.
use ramo_core::git::Commit;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub from_node: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Row {
    pub incoming: bool,
    pub lane: usize,
    pub lanes: usize,
    pub edges: Vec<Edge>,
}

pub fn layout(commits: &[Commit]) -> Vec<Row> {
    let mut lanes: Vec<Option<&str>> = Vec::new();
    commits
        .iter()
        .map(|commit| {
            let existing = lanes
                .iter()
                .position(|oid| *oid == Some(commit.oid.as_str()));
            let lane = existing.unwrap_or_else(|| {
                let lane = lanes
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(lanes.len());
                if lane == lanes.len() {
                    lanes.push(None);
                }
                lanes[lane] = Some(&commit.oid);
                lane
            });
            let before = lanes.clone();
            lanes[lane] = None;
            let mut targets = Vec::new();
            for (index, parent) in commit.parents.iter().enumerate() {
                let target = lanes
                    .iter()
                    .position(|oid| *oid == Some(parent.as_str()))
                    .unwrap_or_else(|| {
                        let target = if index == 0 && lanes[lane].is_none() {
                            lane
                        } else {
                            lanes
                                .iter()
                                .position(Option::is_none)
                                .unwrap_or(lanes.len())
                        };
                        if target == lanes.len() {
                            lanes.push(None);
                        }
                        lanes[target] = Some(parent);
                        target
                    });
                targets.push(target);
            }
            let mut edges: Vec<_> = before
                .iter()
                .enumerate()
                .filter(|(from, oid)| oid.is_some() && *from != lane)
                .map(|(from, _)| Edge {
                    from,
                    to: from,
                    from_node: false,
                })
                .collect();
            edges.extend(targets.into_iter().map(|to| Edge {
                from: lane,
                to,
                from_node: true,
            }));
            let width = before.len().max(lanes.len()).max(lane + 1);
            while lanes.last().is_some_and(Option::is_none) {
                lanes.pop();
            }
            Row {
                incoming: existing.is_some(),
                lane,
                lanes: width,
                edges,
            }
        })
        .collect()
}

/// Painting filters rows after topology is calculated. Edges crossing a hidden
/// interval are never drawn as if the visible rows were directly adjacent.
pub fn adjacent(indices: &[usize], position: usize) -> (bool, bool) {
    let index = indices[position];
    (
        position > 0 && indices[position - 1] + 1 == index,
        position + 1 < indices.len() && index + 1 == indices[position + 1],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(oid: &str, parents: &[&str]) -> Commit {
        Commit {
            oid: oid.into(),
            parents: parents.iter().map(|p| (*p).into()).collect(),
            author: String::new(),
            date: String::new(),
            subject: String::new(),
            refs: String::new(),
        }
    }
    #[test]
    fn agrees_with_released_typescript_fixture_outputs() {
        #[derive(serde::Deserialize)]
        struct Case {
            commits: Vec<Commit>,
            rows: Vec<Row>,
        }
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../fixtures/graph-parity.json")).unwrap();
        assert!(cases.len() >= 5);
        for case in cases {
            assert_eq!(layout(&case.commits), case.rows);
        }
    }
    #[test]
    fn merge_rejoins_and_handles_disconnected_and_octopus_roots() {
        let rows = layout(&[
            c("merge", &["left", "right", "third"]),
            c("left", &["base"]),
            c("right", &["base"]),
            c("third", &["base"]),
            c("base", &[]),
            c("other", &[]),
        ]);
        assert_eq!(
            rows[0].edges.iter().map(|e| e.to).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert!(rows[2].edges.contains(&Edge {
            from: 1,
            to: 0,
            from_node: true
        }));
        assert!(rows[4].edges.is_empty());
        assert!(!rows[5].incoming);
        assert!(rows[5].edges.is_empty());
    }
    #[test]
    fn large_linear_history_and_page_boundaries_are_stable() {
        let commits: Vec<_> = (0..50_000)
            .map(|i| {
                c(
                    &i.to_string(),
                    &if i < 49_999 {
                        vec![(i + 1).to_string()]
                    } else {
                        vec![]
                    }
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                )
            })
            .collect();
        let rows = layout(&commits);
        assert!(rows.iter().all(|r| r.lane == 0 && r.lanes == 1));
        assert_eq!(&layout(&commits[..300]), &rows[..300]);
        assert!(rows[49_999].edges.is_empty());
    }
    #[test]
    fn filtering_does_not_invent_edges_between_unrelated_visible_rows() {
        assert_eq!(adjacent(&[0, 1, 7, 8], 1), (true, false));
        assert_eq!(adjacent(&[0, 1, 7, 8], 2), (false, true));
    }
}
