use std::path::{Path, PathBuf};

/// Fast parallel directory size using jwalk (rayon-backed)
fn parallel_dir_size(path: &Path) -> u64 {
    jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Scan children: list entries with readdir, then size each with jwalk
fn fast_children_sizes(path: &Path) -> Vec<(String, PathBuf, u64, bool)> {
    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.') || name == ".Trash"
            })
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let path = e.path();
                let is_dir = path.is_dir();
                (name, path, is_dir)
            })
            .collect(),
        Err(_) => return Vec::new(),
    };

    let mut results: Vec<(String, PathBuf, u64, bool)> = entries
        .into_iter()
        .map(|(name, path, is_dir)| {
            let size = if is_dir {
                parallel_dir_size(&path)
            } else {
                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            };
            (name, path, size, is_dir)
        })
        .filter(|(_, _, size, _)| *size >= 1_000_000)
        .collect();

    results.sort_by(|a, b| b.2.cmp(&a.2));
    results
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
    /// Load immediate children using parallel jwalk
    pub fn load_children(&mut self) {
        if self.children_loaded || !self.is_dir {
            return;
        }

        let children = fast_children_sizes(&self.path);
        for (name, path, size, is_dir) in children {
            self.children.push(SpaceNode {
                name,
                path,
                size,
                is_dir,
                expanded: false,
                children: Vec::new(),
                children_loaded: false,
            });
        }

        self.children_loaded = true;
    }
}

/// Scan top-level home directories
pub fn scan_home_tree() -> Vec<SpaceNode> {
    let home = dirs::home_dir().unwrap_or_default();
    let children = fast_children_sizes(&home);

    children
        .into_iter()
        .map(|(name, path, size, is_dir)| SpaceNode {
            name,
            path,
            size,
            is_dir,
            expanded: false,
            children: Vec::new(),
            children_loaded: false,
        })
        .collect()
}

/// Visible item for flattened tree rendering
#[derive(Debug, Clone)]
pub struct SpaceVisibleItem {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
    pub tree_path: Vec<usize>,
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
