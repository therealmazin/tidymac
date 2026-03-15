use super::dir_size;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SpaceEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct SpaceNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<SpaceNode>,
    pub children_loaded: bool,
}

impl SpaceNode {
    /// Load immediate children of this directory node
    pub fn load_children(&mut self) {
        if self.children_loaded || !self.is_dir {
            return;
        }

        let read_dir = match std::fs::read_dir(&self.path) {
            Ok(rd) => rd,
            Err(_) => {
                self.children_loaded = true;
                return;
            }
        };

        for entry in read_dir.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/dirs (except .Trash)
            if name.starts_with('.') && name != ".Trash" {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let size = if is_dir {
                dir_size(&entry_path)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            };

            if size > 1_000_000 { // Only show > 1 MB
                self.children.push(SpaceNode {
                    name,
                    path: entry_path,
                    size,
                    is_dir,
                    expanded: false,
                    children: Vec::new(),
                    children_loaded: false,
                });
            }
        }

        self.children.sort_by(|a, b| b.size.cmp(&a.size));
        self.children_loaded = true;
    }
}

/// Scan top-level home directories (initial fast scan)
pub fn scan_home_tree() -> Vec<SpaceNode> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut nodes = Vec::new();

    let read_dir = match std::fs::read_dir(&home) {
        Ok(rd) => rd,
        Err(_) => return nodes,
    };

    for entry in read_dir.filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') && name != ".Trash" {
            continue;
        }

        let is_dir = entry_path.is_dir();
        let size = if is_dir {
            dir_size(&entry_path)
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };

        if size > 1_000_000 {
            nodes.push(SpaceNode {
                name,
                path: entry_path,
                size,
                is_dir,
                expanded: false,
                children: Vec::new(),
                children_loaded: false,
            });
        }
    }

    nodes.sort_by(|a, b| b.size.cmp(&a.size));
    nodes
}

/// Visible item for flattened tree rendering
#[derive(Debug, Clone)]
pub struct SpaceVisibleItem {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
    pub tree_path: Vec<usize>, // indices to navigate to this node in the tree
}

/// Flatten tree into visible items list
pub fn flatten_tree(nodes: &[SpaceNode]) -> Vec<SpaceVisibleItem> {
    let mut items = Vec::new();
    flatten_recursive(nodes, 0, &mut Vec::new(), &mut items);
    items
}

fn flatten_recursive(
    nodes: &[SpaceNode],
    depth: usize,
    path: &mut Vec<usize>,
    items: &mut Vec<SpaceVisibleItem>,
) {
    for (i, node) in nodes.iter().enumerate() {
        path.push(i);
        items.push(SpaceVisibleItem {
            name: node.name.clone(),
            size: node.size,
            is_dir: node.is_dir,
            expanded: node.expanded,
            depth,
            tree_path: path.clone(),
        });

        if node.expanded && !node.children.is_empty() {
            flatten_recursive(&node.children, depth + 1, path, items);
        }
        path.pop();
    }
}

/// Navigate to a node in the tree by its path indices
pub fn get_node_mut<'a>(tree: &'a mut [SpaceNode], tree_path: &[usize]) -> Option<&'a mut SpaceNode> {
    if tree_path.is_empty() {
        return None;
    }
    let first = *tree_path.first()?;
    if first >= tree.len() {
        return None;
    }
    let mut node = &mut tree[first];
    for &idx in &tree_path[1..] {
        if idx >= node.children.len() {
            return None;
        }
        node = &mut node.children[idx];
    }
    Some(node)
}

// Keep for backward compat with ScanMessage::SpaceResults
pub fn scan_dir(path: &std::path::Path) -> Vec<SpaceEntry> {
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read_dir.filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            if let Ok(meta) = entry.metadata() {
                if meta.len() > 1_000_000 {
                    entries.push(SpaceEntry {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry_path,
                        size: meta.len(),
                    });
                }
            }
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != ".Trash" {
            continue;
        }

        let size = dir_size(&entry_path);
        if size > 1_000_000 {
            entries.push(SpaceEntry {
                name,
                path: entry_path,
                size,
            });
        }
    }

    entries.sort_by(|a, b| b.size.cmp(&a.size));
    entries
}
