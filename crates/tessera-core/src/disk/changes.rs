//! Largest increases in allocated space among the current map's directory items.

use std::cmp::Ordering;
use std::time::SystemTime;

use super::{Category, DiskIndex, NodeId, Overview};

const MAX_ROWS: usize = 50;

#[derive(Clone, Debug)]
pub struct DiskChanges {
    pub previous_at: SystemTime,
    pub current_at: SystemTime,
    pub rows: Vec<GrowthRow>,
}

#[derive(Clone, Debug)]
pub struct GrowthRow {
    /// Node in the current index, for opening this item on the map.
    pub id: NodeId,
    pub category: Category,
    pub label: String,
    pub display_path: String,
    pub growth: u64,
    /// The previous index folded this path into its parent's unindexed bytes.
    pub at_least: bool,
}

/// Compare two scans with the same scope and readable paths. The semantic
/// overview supplies non-overlapping candidates, so a project and its build
/// directory cannot both claim the same increase.
pub fn compare(previous: &DiskIndex, current: &DiskIndex, overview: &Overview) -> DiskChanges {
    let mut rows: Vec<GrowthRow> = Vec::with_capacity(MAX_ROWS);
    let mut components = Vec::new();
    for item in overview.groups.iter().flat_map(|group| &group.items) {
        let Some(id) = item.node else { continue };
        let node = current.node(id);
        if !node.is_dir() {
            continue;
        }
        let Some((before, at_least)) = previous_size(previous, current, id, &mut components) else {
            continue;
        };
        let growth = node.size.saturating_sub(before);
        if growth == 0 {
            continue;
        }

        // Keep at most MAX_ROWS names. Allocating a path for tie-breaking is
        // only necessary for identically labelled items with identical growth.
        let position = rows.partition_point(|row| {
            growth
                .cmp(&row.growth)
                .then_with(|| row.label.cmp(&item.label))
                .then_with(|| row.category.cmp(&item.category))
                .then_with(|| current.path(row.id).cmp(&current.path(id)))
                != Ordering::Greater
        });
        if position < MAX_ROWS {
            if rows.len() == MAX_ROWS {
                rows.pop();
            }
            rows.insert(
                position,
                GrowthRow {
                    id,
                    category: item.category,
                    label: item.label.clone(),
                    display_path: String::new(),
                    growth,
                    at_least,
                },
            );
        }
    }
    for row in &mut rows {
        row.display_path = current.display_path(row.id);
    }
    DiskChanges {
        previous_at: previous.scanned_at,
        current_at: current.scanned_at,
        rows,
    }
}

/// Resolve just this path in the old index. If a component was absent, the
/// closest indexed parent's unindexed bytes bound everything that could have
/// existed beneath it, even when the child cap was reached.
fn previous_size<'a>(
    previous: &DiskIndex,
    current: &'a DiskIndex,
    mut id: NodeId,
    components: &mut Vec<&'a str>,
) -> Option<(u64, bool)> {
    components.clear();
    while let Some(parent) = current.parent(id) {
        let name = current.name(id);
        // Index names use lossy UTF-8. Such a component cannot safely match
        // a path in another scan, since multiple byte sequences can alias.
        if name.contains(char::REPLACEMENT_CHARACTER) {
            return None;
        }
        components.push(name);
        id = parent;
    }
    let mut old = previous.root();
    for name in components.iter().rev() {
        match previous.child_by_name(old, name) {
            Some(child) => old = child,
            None => {
                let upper_bound = if previous.node(old).is_dir() {
                    previous.remainder(old).0
                } else {
                    0
                };
                return Some((upper_bound, upper_bound != 0));
            }
        }
    }
    Some((previous.node(old).size, false))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::UNIX_EPOCH;

    use super::*;
    use crate::disk::index::IndexStats;
    use crate::disk::scan::Tmp;
    use crate::disk::{Kind, Lang, flags, overview};

    fn node(name: &str, kind: Kind, size: u64, children: Vec<Tmp>) -> Tmp {
        let mut flags = if kind.is_file() { 0 } else { flags::DIR };
        if children
            .iter()
            .any(|c| c.kind == Kind::Project || c.flags & flags::HAS_PROJECT != 0)
        {
            flags |= flags::HAS_PROJECT;
        }
        Tmp {
            name: name.as_bytes().into(),
            kind,
            lang: Lang::Unknown,
            flags,
            size,
            files: 1,
            mtime: 0,
            children,
        }
    }

    fn index(children: Vec<Tmp>, unindexed: u64) -> DiskIndex {
        let size = children.iter().map(|child| child.size).sum::<u64>() + unindexed;
        DiskIndex::from_tree(
            node("root", Kind::Folder, size, children),
            "/root".into(),
            false,
            UNIX_EPOCH,
            IndexStats::default(),
        )
    }

    fn changes(previous: &DiskIndex, current: &DiskIndex) -> Vec<GrowthRow> {
        let overview = overview::build(current, current.total(), Path::new("/root"));
        compare(previous, current, &overview).rows
    }

    #[test]
    fn ranks_directory_growth_by_path_across_reordered_scans() {
        let project = |parent, size| {
            node(
                parent,
                Kind::Folder,
                size,
                vec![node("app", Kind::Project, size, vec![])],
            )
        };
        let previous = index(vec![project("b", 100), project("a", 30)], 0);
        let current = index(
            vec![
                node("movie", Kind::Video, 500, vec![]),
                project("a", 130),
                project("b", 110),
            ],
            600,
        );
        let rows = changes(&previous, &current);
        assert_eq!(rows.len(), 2);
        assert_eq!(current.path(rows[0].id), Path::new("/root/a/app"));
        assert_eq!(rows[0].display_path, "/root/a/app");
        assert_eq!(rows[0].category, Category::Projects);
        assert_eq!(rows[0].growth, 100);
        assert_eq!(rows[1].growth, 10);
        assert!(rows.iter().all(|row| row.label == "app" && !row.at_least));
    }

    #[test]
    fn missing_paths_use_the_closest_old_parents_remainder_as_a_bound() {
        let previous = index(vec![node("dev", Kind::Folder, 70, vec![])], 1000);
        let current = index(
            vec![node(
                "dev",
                Kind::Folder,
                165,
                vec![
                    node("large", Kind::Project, 100, vec![]),
                    node("small", Kind::Project, 65, vec![]),
                ],
            )],
            1000,
        );
        let rows = changes(&previous, &current);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "large");
        assert_eq!(rows[0].growth, 30);
        assert!(rows[0].at_least);
    }

    #[test]
    fn caps_tied_rows_in_name_order_and_skips_unreliable_paths() {
        let previous = index(vec![], 0);
        let mut children: Vec<_> = (0..52)
            .rev()
            .map(|n| node(&format!("{n:02}"), Kind::Folder, 20, vec![]))
            .collect();
        children.push(node("invalid�", Kind::Folder, 100, vec![]));
        let current = index(children, 0);
        let rows = changes(&previous, &current);
        assert_eq!(rows.len(), MAX_ROWS);
        assert_eq!(rows.first().unwrap().label, "00");
        assert_eq!(rows.last().unwrap().label, "49");
        assert!(rows.iter().all(|row| !row.at_least));
    }
}
