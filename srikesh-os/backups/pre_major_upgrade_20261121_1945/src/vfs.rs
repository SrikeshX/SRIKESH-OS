//! SRIKESH OS - Virtual File System (VFS)
//!
//! Provides a hierarchical Unix-like filesystem structure with support for:
//! - Directories and nested paths (/home/srikesh/, /etc/, /packages/)
//! - File creation, reading, writing, and deletion
//! - Integration with persistent storage and system configuration

use crate::serial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    StorageFull,
    InvalidPath,
    ReadOnly,
}

pub const MAX_NODES: usize = 64;
pub const MAX_FILE_SIZE: usize = 2048;

#[derive(Clone, Copy)]
pub struct VfsNode {
    pub in_use: bool,
    pub parent_idx: usize,
    pub name: [u8; 32],
    pub name_len: usize,
    pub is_dir: bool,
    pub data: [u8; MAX_FILE_SIZE],
    pub data_len: usize,
}

impl VfsNode {
    pub const fn empty() -> Self {
        Self {
            in_use: false,
            parent_idx: 0,
            name: [0; 32],
            name_len: 0,
            is_dir: false,
            data: [0; MAX_FILE_SIZE],
            data_len: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("<invalid>")
    }
}

pub struct VirtualFileSystem {
    nodes: [VfsNode; MAX_NODES],
    root_idx: usize,
}

impl VirtualFileSystem {
    pub const fn new() -> Self {
        Self {
            nodes: [VfsNode::empty(); MAX_NODES],
            root_idx: 0,
        }
    }

    /// Initialize default system directory tree and files
    pub fn init_defaults(&mut self) {
        // Node 0: Root directory "/"
        let mut root_name = [0u8; 32];
        root_name[0] = b'/';
        self.nodes[0] = VfsNode {
            in_use: true,
            parent_idx: 0,
            name: root_name,
            name_len: 1,
            is_dir: true,
            data: [0; MAX_FILE_SIZE],
            data_len: 0,
        };

        // Create core directories
        let _ = self.create_dir_internal(0, "home");
        let _ = self.create_dir_internal(0, "etc");
        let _ = self.create_dir_internal(0, "packages");
        let _ = self.create_dir_internal(0, "sys");

        // Create /home/srikesh/
        if let Some(home_idx) = self.find_child(0, "home") {
            let _ = self.create_dir_internal(home_idx, "srikesh");
            if let Some(user_idx) = self.find_child(home_idx, "srikesh") {
                let _ = self.create_file_internal(
                    user_idx,
                    "welcome.txt",
                    b"Welcome to SRIKESH OS!\n\nA modern, portable operating system powered by a genuine Rust microkernel.\nFeatures preemptive multitasking, high-res double-buffered GUI, and zero-write host drive protection.\n",
                );
                let _ = self.create_file_internal(
                    user_idx,
                    "notes.txt",
                    b"SRIKESH OS Architecture:\n- Microkernel: Rust no_std x86-64\n- Preemptive Quantum: 5 PIT Ticks (~50 ms)\n- Host Security Guard: Enforced zero-write protection\n- Filesystem: Hierarchical VFS + USB Persistence\n",
                );
            }
        }

        // Create /etc/os-release and /etc/settings.cfg
        if let Some(etc_idx) = self.find_child(0, "etc") {
            let _ = self.create_file_internal(
                etc_idx,
                "os-release",
                b"NAME=\"SRIKESH OS\"\nVERSION=\"0.2.0-Aurora\"\nID=srikesh-os\nPRETTY_NAME=\"SRIKESH OS (x86-64 Rust Microkernel)\"\nBUILD=2026.11\nAUTHOR=\"SRIKESH S\"\n",
            );
            let _ = self.create_file_internal(
                etc_idx,
                "settings.cfg",
                b"theme=aurora_cyberpunk\nwallpaper=cyberpunk_neon\nresolution=1280x720\nuser=srikesh\npersistence=enabled\nboot_count=1\n",
            );
        }

        serial::write_str("[VFS] Virtual File System initialized with standard system hierarchy.\n");
    }

    fn alloc_node(&mut self) -> Option<usize> {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if !node.in_use {
                *node = VfsNode::empty();
                node.in_use = true;
                return Some(i);
            }
        }
        None
    }

    fn find_child(&self, parent_idx: usize, name: &str) -> Option<usize> {
        for (i, node) in self.nodes.iter().enumerate() {
            if node.in_use && node.parent_idx == parent_idx && i != 0 {
                if node.name_str() == name {
                    return Some(i);
                }
            }
        }
        None
    }

    fn create_file_internal(&mut self, parent_idx: usize, name: &str, content: &[u8]) -> Result<usize, FsError> {
        if self.find_child(parent_idx, name).is_some() {
            return Err(FsError::AlreadyExists);
        }
        let idx = self.alloc_node().ok_or(FsError::StorageFull)?;
        let node = &mut self.nodes[idx];
        node.parent_idx = parent_idx;
        node.is_dir = false;

        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(31);
        node.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        node.name_len = copy_len;

        let copy_data_len = content.len().min(MAX_FILE_SIZE);
        node.data[..copy_data_len].copy_from_slice(&content[..copy_data_len]);
        node.data_len = copy_data_len;

        Ok(idx)
    }

    fn create_dir_internal(&mut self, parent_idx: usize, name: &str) -> Result<usize, FsError> {
        if self.find_child(parent_idx, name).is_some() {
            return Err(FsError::AlreadyExists);
        }
        let idx = self.alloc_node().ok_or(FsError::StorageFull)?;
        let node = &mut self.nodes[idx];
        node.parent_idx = parent_idx;
        node.is_dir = true;

        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(31);
        node.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        node.name_len = copy_len;

        Ok(idx)
    }

    /// Resolve an absolute path like "/home/srikesh/welcome.txt" to a node index
    pub fn resolve_path(&self, path: &str) -> Result<usize, FsError> {
        if path == "/" {
            return Ok(self.root_idx);
        }
        let clean = path.trim_start_matches('/');
        let mut cur = self.root_idx;

        for part in clean.split('/') {
            if part.is_empty() { continue; }
            match self.find_child(cur, part) {
                Some(child) => cur = child,
                None => return Err(FsError::NotFound),
            }
        }
        Ok(cur)
    }

    /// Resolve parent directory index and base filename from a path
    fn split_path<'a>(&self, path: &'a str) -> Result<(usize, &'a str), FsError> {
        let clean = path.trim_end_matches('/');
        if let Some(last_slash) = clean.rfind('/') {
            let parent_path = if last_slash == 0 { "/" } else { &clean[..last_slash] };
            let filename = &clean[last_slash + 1..];
            let parent_idx = self.resolve_path(parent_path)?;
            Ok((parent_idx, filename))
        } else {
            Ok((self.root_idx, clean))
        }
    }

    pub fn read_file(&self, path: &str) -> Result<&[u8], FsError> {
        let idx = self.resolve_path(path)?;
        let node = &self.nodes[idx];
        if node.is_dir {
            return Err(FsError::IsADirectory);
        }
        Ok(&node.data[..node.data_len])
    }

    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        match self.resolve_path(path) {
            Ok(idx) => {
                let node = &mut self.nodes[idx];
                if node.is_dir {
                    return Err(FsError::IsADirectory);
                }
                let copy_len = content.len().min(MAX_FILE_SIZE);
                node.data[..copy_len].copy_from_slice(&content[..copy_len]);
                node.data_len = copy_len;
                Ok(())
            }
            Err(FsError::NotFound) => {
                let (parent_idx, name) = self.split_path(path)?;
                self.create_file_internal(parent_idx, name, content)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let (parent_idx, name) = self.split_path(path)?;
        self.create_file_internal(parent_idx, name, content)?;
        Ok(())
    }

    pub fn create_dir(&mut self, path: &str) -> Result<(), FsError> {
        let (parent_idx, name) = self.split_path(path)?;
        self.create_dir_internal(parent_idx, name)?;
        Ok(())
    }

    pub fn list_dir(&self, path: &str) -> Result<([(&str, bool, usize); 16], usize), FsError> {
        let idx = self.resolve_path(path)?;
        if !self.nodes[idx].is_dir {
            return Err(FsError::NotADirectory);
        }

        let mut results = [("", false, 0); 16];
        let mut count = 0;

        for (i, node) in self.nodes.iter().enumerate() {
            if node.in_use && node.parent_idx == idx && i != 0 {
                if count < results.len() {
                    results[count] = (node.name_str(), node.is_dir, node.data_len);
                    count += 1;
                }
            }
        }
        Ok((results, count))
    }

    pub fn delete_file(&mut self, path: &str) -> Result<(), FsError> {
        let idx = self.resolve_path(path)?;
        if idx == self.root_idx {
            return Err(FsError::InvalidPath);
        }
        let is_dir = self.nodes[idx].is_dir;
        if is_dir {
            // Check if directory has children
            for child in &self.nodes {
                if child.in_use && child.parent_idx == idx {
                    return Err(FsError::DirectoryNotEmpty);
                }
            }
        }
        self.nodes[idx].in_use = false;
        Ok(())
    }
}

pub static VFS: spin::Mutex<VirtualFileSystem> = spin::Mutex::new(VirtualFileSystem::new());
