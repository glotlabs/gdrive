use crate::common::file_info::FileInfo;
use crate::common::id_gen;
use crate::common::id_gen::IdGen;
use async_recursion::async_recursion;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileTree {
    pub root: Folder,
}

impl FileTree {
    pub async fn from_path<'a>(path: &PathBuf, ids: &mut IdGen<'a>) -> Result<FileTree, Error> {
        let canonical_path = path
            .canonicalize()
            .map_err(|err| Error::CanonicalizePath(path.clone(), err))?;

        let root = Folder::from_path(&canonical_path, None, ids).await?;
        Ok(FileTree { root })
    }

    pub fn folders(&self) -> Vec<Folder> {
        let mut folders = vec![];

        folders.push(self.root.clone());
        let child_folders = self.root.folders_recursive();
        folders.extend(child_folders);

        folders.sort_by(|a, b| {
            let parent_count_a = a.ancestor_count();
            let parent_count_b = b.ancestor_count();

            if parent_count_a == parent_count_b {
                a.name.cmp(&b.name)
            } else {
                parent_count_a.cmp(&parent_count_b)
            }
        });

        folders
    }

    pub fn info(&self) -> TreeInfo {
        let mut file_count = 0;
        let mut folder_count = 0;
        let mut total_file_size = 0;

        for folder in self.folders() {
            folder_count += 1;

            for file in folder.files() {
                file_count += 1;
                total_file_size += file.size as u128;
            }
        }

        TreeInfo {
            file_count,
            folder_count,
            total_file_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeInfo {
    pub file_count: u128,
    pub folder_count: u128,
    pub total_file_size: u128,
}

#[derive(Debug, Clone)]
pub enum Node {
    FolderNode(Folder),
    FileNode(File),
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub name: String,
    pub path: PathBuf,
    pub parent: Option<Box<Folder>>,
    pub children: Vec<Node>,
    pub drive_id: String,
}

impl Folder {
    #[async_recursion]
    pub async fn from_path<'a>(
        path: &PathBuf,
        parent: Option<&'async_recursion Folder>,
        ids: &mut IdGen<'a>,
    ) -> Result<Folder, Error> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(Error::InvalidPath(path.clone()))?;

        let drive_id = ids.next().await.map_err(Error::GetId)?;

        let mut folder = Folder {
            name,
            path: path.clone(),
            parent: parent.map(|folder| Box::new(folder.clone())),
            children: Vec::new(),
            drive_id,
        };

        let entries = fs::read_dir(path).map_err(Error::ReadDir)?;
        let mut children = Vec::new();

        for e in entries {
            let entry = e.map_err(Error::ReadDirEntry)?;
            let path = entry.path();

            if path.is_dir() {
                let folder = Folder::from_path(&path, Some(&folder), ids).await?;
                let node = Node::FolderNode(folder);
                children.push(node);
            } else if path.is_file() {
                let file = File::from_path(&path, &folder, ids).await?;
                let node = Node::FileNode(file);
                children.push(node);
            } else if path.is_file() {
                return Err(Error::IsSymlink(path.clone()));
            } else {
                return Err(Error::UnknownFileType(path.clone()));
            }
        }

        folder.children = children;

        Ok(folder)
    }

    pub fn files(&self) -> Vec<File> {
        let mut files = vec![];

        for child in &self.children {
            if let Node::FileNode(file) = child {
                files.push(file.clone());
            }
        }

        files.sort_by(|a, b| a.name.cmp(&b.name));

        files
    }

    pub fn relative_path(&self) -> PathBuf {
        let mut root_path = get_root_folder(self).path;
        root_path.pop();
        self.path.strip_prefix(root_path).unwrap().to_path_buf()
    }

    pub fn folders_recursive(&self) -> Vec<Folder> {
        Folder::collect_folders_recursive(&self)
    }

    pub fn ancestor_count(&self) -> usize {
        let mut count = 0;
        let mut parent = self.parent.clone();

        while let Some(folder) = parent {
            count += 1;
            parent = folder.parent.clone();
        }

        count
    }

    fn collect_folders_recursive(folder: &Folder) -> Vec<Folder> {
        let mut folders = vec![];

        folder.children.iter().for_each(|child| {
            if let Node::FolderNode(folder) = child {
                folders.push(folder.clone());
                let child_folders = Folder::collect_folders_recursive(folder);
                folders.extend(child_folders);
            }
        });

        folders
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub mime_type: mime::Mime,
    pub parent: Folder,
    pub drive_id: String,
}

impl File {
    pub async fn from_path<'a>(
        path: &PathBuf,
        parent: &Folder,
        ids: &mut IdGen<'a>,
    ) -> Result<File, Error> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(Error::InvalidPath(path.clone()))?;

        let os_file = fs::File::open(path).map_err(|err| Error::OpenFile(path.clone(), err))?;
        let size = os_file.metadata().map(|m| m.len()).unwrap_or(0);
        let mime_type = mime_guess::from_path(path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let drive_id = ids.next().await.map_err(Error::GetId)?;

        let file = File {
            name,
            path: path.clone(),
            size,
            mime_type,
            parent: parent.clone(),
            drive_id,
        };

        Ok(file)
    }

    pub fn relative_path(&self) -> PathBuf {
        let mut root_path = get_root_folder(&self.parent).path;
        root_path.pop();
        self.path.strip_prefix(root_path).unwrap().to_path_buf()
    }

    pub fn info(&self, parents: Option<Vec<String>>) -> FileInfo {
        FileInfo {
            name: self.name.clone(),
            size: self.size,
            mime_type: self.mime_type.clone(),
            parents,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    CanonicalizePath(PathBuf, io::Error),
    ReadDir(io::Error),
    ReadDirEntry(io::Error),
    OpenFile(PathBuf, io::Error),
    GetId(id_gen::Error),
    InvalidPath(PathBuf),
    IsSymlink(PathBuf),
    UnknownFileType(PathBuf),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CanonicalizePath(path, e) => write!(
                f,
                "Failed to get canonical path of {}: {}",
                path.display(),
                e
            ),
            Error::ReadDir(e) => write!(f, "Error reading directory: '{}'", e),
            Error::ReadDirEntry(e) => write!(f, "Error reading directory entry: {}", e),
            Error::OpenFile(path, e) => {
                write!(f, "Failed to open file '{}': {}", path.display(), e)
            }
            Error::GetId(e) => write!(f, "Error getting id: {}", e),
            Error::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            Error::IsSymlink(path) => write!(f, "Path is symlink: {}", path.display()),
            Error::UnknownFileType(path) => write!(f, "Unknown file type: {}", path.display()),
        }
    }
}

fn get_root_folder(folder: &Folder) -> Folder {
    let mut root_candidate = Some(folder.clone());

    while let Some(folder) = root_candidate {
        if folder.parent.is_none() {
            return folder.clone();
        }

        root_candidate = folder.parent.map(|folder| *folder.clone());
    }

    folder.clone()
}
