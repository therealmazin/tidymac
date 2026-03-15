use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

// macOS constants not in libc crate
const ATTR_CMN_ERROR: u32 = 0x20000000;
const VREG: u32 = 1;
const VDIR: u32 = 2;

struct DirContents {
    file_sizes: Vec<u64>, // allocation sizes of regular files
    subdirs: Vec<String>,
}

/// Read directory contents using getattrlistbulk — one syscall per directory
/// instead of individual stat() calls per file. ~6x faster than walkdir/jwalk.
fn read_dir_bulk(path: &str) -> Option<DirContents> {
    let c_path = CString::new(path).ok()?;
    let dirfd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
    if dirfd == -1 {
        return None;
    }

    let mut attrlist = libc::attrlist {
        bitmapcount: libc::ATTR_BIT_MAP_COUNT as u16,
        reserved: 0,
        commonattr: libc::ATTR_CMN_RETURNED_ATTRS
            | libc::ATTR_CMN_NAME
            | ATTR_CMN_ERROR
            | libc::ATTR_CMN_OBJTYPE,
        volattr: 0,
        dirattr: 0,
        fileattr: libc::ATTR_FILE_ALLOCSIZE,
        forkattr: 0,
    };

    let mut attrbuf = [0u8; 128 * 1024];
    let mut file_sizes = Vec::new();
    let mut subdirs = Vec::new();

    loop {
        let retcount = unsafe {
            libc::getattrlistbulk(
                dirfd,
                &mut attrlist as *mut libc::attrlist as *mut libc::c_void,
                attrbuf.as_mut_ptr() as *mut libc::c_void,
                attrbuf.len(),
                0,
            )
        };

        if retcount <= 0 {
            break;
        }

        let mut entry_ptr = attrbuf.as_ptr();
        for _ in 0..retcount {
            unsafe {
                let entry_length = std::ptr::read_unaligned(entry_ptr as *const u32);
                let mut field_ptr = entry_ptr.add(std::mem::size_of::<u32>());

                // Returned attributes bitmask
                let returned_attrs =
                    std::ptr::read_unaligned(field_ptr as *const libc::attribute_set_t);
                field_ptr = field_ptr.add(std::mem::size_of::<libc::attribute_set_t>());

                // Extract filename
                let mut filename: Option<String> = None;
                if returned_attrs.commonattr & libc::ATTR_CMN_NAME != 0 {
                    let name_start = field_ptr;
                    let name_info =
                        std::ptr::read_unaligned(field_ptr as *const libc::attrreference_t);
                    field_ptr = field_ptr.add(std::mem::size_of::<libc::attrreference_t>());
                    let name_ptr = name_start.add(name_info.attr_dataoffset as usize);

                    if name_info.attr_length > 0 {
                        let name_slice = std::slice::from_raw_parts(
                            name_ptr,
                            (name_info.attr_length - 1) as usize,
                        );
                        if let Ok(name_str) = std::str::from_utf8(name_slice) {
                            if name_str != "." && name_str != ".." {
                                filename = Some(name_str.to_string());
                            }
                        }
                    }
                }

                // Check for errors
                if returned_attrs.commonattr & ATTR_CMN_ERROR != 0 {
                    let error_code = std::ptr::read_unaligned(field_ptr as *const u32);
                    field_ptr = field_ptr.add(std::mem::size_of::<u32>());
                    if error_code != 0 {
                        entry_ptr = entry_ptr.add(entry_length as usize);
                        continue;
                    }
                }

                // Get object type
                let obj_type = if returned_attrs.commonattr & libc::ATTR_CMN_OBJTYPE != 0 {
                    let obj_type = std::ptr::read_unaligned(field_ptr as *const u32);
                    field_ptr = field_ptr.add(std::mem::size_of::<u32>());
                    obj_type
                } else {
                    0
                };

                match obj_type {
                    VREG => {
                        // Regular file — get allocation size
                        if returned_attrs.fileattr & libc::ATTR_FILE_ALLOCSIZE != 0 {
                            let alloc_size =
                                std::ptr::read_unaligned(field_ptr as *const i64);
                            if alloc_size > 0 {
                                file_sizes.push(alloc_size as u64);
                            }
                        }
                    }
                    VDIR => {
                        if let Some(name) = filename {
                            subdirs.push(name);
                        }
                    }
                    _ => {}
                }

                entry_ptr = entry_ptr.add(entry_length as usize);
            }
        }
    }

    unsafe {
        libc::close(dirfd);
    }

    Some(DirContents {
        file_sizes,
        subdirs,
    })
}

/// Recursively calculate directory size using getattrlistbulk + rayon
fn calculate_size(path: &str) -> u64 {
    let contents = match read_dir_bulk(path) {
        Some(c) => c,
        None => return 0,
    };

    // Sum file sizes in this directory
    let file_total: u64 = contents.file_sizes.iter().sum();

    // Recurse into subdirectories in parallel
    let subdir_total: u64 = contents
        .subdirs
        .par_iter()
        .map(|subdir| {
            let subdir_path = Path::new(path).join(subdir);
            calculate_size(&subdir_path.to_string_lossy())
        })
        .sum();

    file_total + subdir_total
}

// --- SpaceNode tree ---

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
    pub fn load_children_from_cache(&mut self, cache: &HashMap<PathBuf, u64>) {
        if self.children_loaded || !self.is_dir {
            return;
        }
        self.children = build_children(&self.path, cache);
        self.children_loaded = true;
    }
}

fn build_children(path: &Path, dir_sizes: &HashMap<PathBuf, u64>) -> Vec<SpaceNode> {
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut nodes: Vec<SpaceNode> = read_dir
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Skip . and .. but show all other entries including hidden ones
            if name == "." || name == ".." {
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
                children_loaded: false,
            })
        })
        .collect();

    nodes.sort_by(|a, b| b.size.cmp(&a.size));
    nodes
}

/// Single-pass scan using getattrlistbulk + rayon.
/// Walks entire home, accumulates sizes per directory for instant drill-down.
pub fn scan_home_tree_with_cache() -> (Vec<SpaceNode>, HashMap<PathBuf, u64>) {
    let home = dirs::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy().to_string();

    // Use a rayon thread pool with large stack size for deep recursion
    let pool = rayon::ThreadPoolBuilder::new()
        .stack_size(16 * 1024 * 1024) // 16MB stack per thread
        .num_threads(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4))
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    let dir_sizes = pool.install(|| scan_dir_sizes_recursive(&home_str, &home));

    let nodes = build_children(&home, &dir_sizes);
    (nodes, dir_sizes)
}

/// Recursively scan directory sizes using getattrlistbulk.
/// Returns a map of directory path → total size (including all descendants).
fn scan_dir_sizes_recursive(path: &str, home: &Path) -> HashMap<PathBuf, u64> {
    let sizes = parking_lot::Mutex::new(HashMap::new());
    scan_dir_recursive_inner(path, home, &sizes);
    sizes.into_inner()
}

fn scan_dir_recursive_inner(
    path: &str,
    home: &Path,
    sizes: &parking_lot::Mutex<HashMap<PathBuf, u64>>,
) -> u64 {
    let contents = match read_dir_bulk(path) {
        Some(c) => c,
        None => return 0,
    };

    let file_total: u64 = contents.file_sizes.iter().sum();

    let subdir_total: u64 = contents
        .subdirs
        .par_iter()
        .map(|subdir| {
            let subdir_path = Path::new(path).join(subdir);
            let subdir_str = subdir_path.to_string_lossy().to_string();
            let sub_size = scan_dir_recursive_inner(&subdir_str, home, sizes);

            // Store this subdirectory's size in the cache
            sizes.lock().insert(subdir_path, sub_size);

            sub_size
        })
        .sum();

    let total = file_total + subdir_total;

    // Store this directory's size
    sizes.lock().insert(PathBuf::from(path), total);

    total
}

// --- Tree flattening and navigation ---

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_space_scan() {
        let start = Instant::now();
        let (nodes, cache) = scan_home_tree_with_cache();
        let elapsed = start.elapsed();

        println!("\n=== Space Lens Benchmark ===");
        println!("Scan time: {:.1}s", elapsed.as_secs_f64());
        println!("Top-level entries: {}", nodes.len());
        println!("Cached directories: {}", cache.len());
        for node in nodes.iter().take(10) {
            println!("  {:<25} {:>10.1} GiB", node.name, node.size as f64 / 1_073_741_824.0);
        }
        println!("===========================\n");

        assert!(!nodes.is_empty(), "Should find directories");
        assert!(elapsed.as_secs() < 60, "Scan should complete in under 60 seconds");
    }
}
