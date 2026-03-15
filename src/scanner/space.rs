use std::path::{Path, PathBuf};

/// Fast directory size using `du -s -k` (much faster than walkdir)
fn fast_dir_size(path: &Path) -> u64 {
    let output = std::process::Command::new("du")
        .args(["-s", "-k", &path.to_string_lossy()])
        .stderr(std::process::Stdio::null())
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            // du output: "12345\t/path/to/dir"
            s.split('\t')
                .next()
                .and_then(|n| n.trim().parse::<u64>().ok())
                .map(|kb| kb * 1024)
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// Scan children by listing dir entries then getting size of each with `du -sk`
fn fast_children_sizes(path: &Path) -> Vec<(String, PathBuf, u64, bool)> {
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut entries: Vec<(String, PathBuf, bool)> = Vec::new();
    for entry in read_dir.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != ".Trash" {
            continue;
        }
        let entry_path = entry.path();
        let is_dir = entry_path.is_dir();
        entries.push((name, entry_path, is_dir));
    }

    let mut results: Vec<(String, PathBuf, u64, bool)> = Vec::new();

    for (name, entry_path, is_dir) in entries {
        let size = if is_dir {
            fast_dir_size(&entry_path)
        } else {
            std::fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0)
        };

        if size >= 1_000_000 {
            results.push((name, entry_path, size, is_dir));
        }
    }

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
    /// Load immediate children using fast du command
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

/// Scan top-level home directories using fast du
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
