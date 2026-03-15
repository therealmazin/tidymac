use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// Build SpaceNode children for a directory using pre-computed sizes
fn build_children(path: &Path, dir_sizes: &HashMap<PathBuf, u64>) -> Vec<SpaceNode> {
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut nodes: Vec<SpaceNode> = read_dir
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name != ".Trash" {
                return None;
            }

            let entry_path = e.path();
            let is_dir = entry_path.is_dir();

            let size = if is_dir {
                dir_sizes.get(&entry_path).copied().unwrap_or(0)
            } else {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            };

            if size < 1_000_000 {
                return None;
            }

            Some(SpaceNode {
                name,
                path: entry_path,
                size,
                is_dir,
                expanded: false,
                children: Vec::new(),
                // Children are pre-computable from dir_sizes — we'll load them lazily
                // but sizes are already known so expand is instant
                children_loaded: false,
            })
        })
        .collect();

    nodes.sort_by(|a, b| b.size.cmp(&a.size));
    nodes
}

/// Single-pass scan: walk entire home once, accumulate sizes to all ancestors.
/// Returns tree + size cache for instant drill-down.
pub fn scan_home_tree_with_cache() -> (Vec<SpaceNode>, HashMap<PathBuf, u64>) {
    let home = dirs::home_dir().unwrap_or_default();

    let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();

    for entry in jwalk::WalkDir::new(&home)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if size == 0 {
            continue;
        }

        let file_path = entry.path();
        let mut current = file_path.as_path();
        while let Some(parent) = current.parent() {
            if parent < home.as_path() {
                break;
            }
            *dir_sizes.entry(parent.to_path_buf()).or_insert(0) += size;
            current = parent;
        }
    }

    let nodes = build_children(&home, &dir_sizes);
    (nodes, dir_sizes)
}

impl SpaceNode {
    /// Load children using cached sizes (instant — no disk scan)
    pub fn load_children_from_cache(&mut self, cache: &HashMap<PathBuf, u64>) {
        if self.children_loaded || !self.is_dir {
            return;
        }
        self.children = build_children(&self.path, cache);
        self.children_loaded = true;
    }
}

// --- Tree flattening and navigation (unchanged) ---

#[derive(Debug, Clone)]
pub struct SpaceVisibleItem {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
    pub tree_path: Vec<usize>,
}

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
