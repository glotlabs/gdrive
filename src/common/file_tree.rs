use crate::common::file_info::FileInfo;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Error {}

#[derive(Debug, Clone)]
pub struct FileTree {
    pub root: Folder,
}

impl FileTree {
    pub fn from_path(path: &PathBuf) -> Result<FileTree, Error> {
        let canonical_path = path.canonicalize().unwrap();
        let root = Folder::from_path(&canonical_path, None).unwrap();
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
}

impl Folder {
    pub fn from_path(path: &PathBuf, parent: Option<&Folder>) -> Result<Folder, Error> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap();
        //.ok_or(Error::InvalidFilePath(path.clone()))?;

        let mut folder = Folder {
            name,
            path: path.clone(),
            parent: parent.map(|folder| Box::new(folder.clone())),
            children: Vec::new(),
        };

        let entries = fs::read_dir(path).unwrap();

        folder.children = entries
            .into_iter()
            .map(|e| {
                let entry = e.unwrap();

                let path = entry.path();

                if path.is_dir() {
                    let folder = Folder::from_path(&path, Some(&folder)).unwrap();
                    Node::FolderNode(folder)
                } else if path.is_file() {
                    let file = File::from_path(&path, &folder).unwrap();
                    Node::FileNode(file)
                } else {
                    // TODO
                    panic!("Invalid path: {}", path.display());
                }
            })
            .collect();

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

    fn root_path(&self) -> Option<PathBuf> {
        let mut parent = self.parent.clone();

        while let Some(folder) = parent {
            if folder.parent.is_none() {
                return Some(folder.path.clone());
            }

            parent = folder.parent.clone();
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub mime_type: mime::Mime,
    pub parent: Box<Node>,
}

impl File {
    pub fn from_path(path: &PathBuf, parent: &Folder) -> Result<File, Error> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap();
        //.ok_or(Error::InvalidFilePath(path.clone()))?;

        let os_file = fs::File::open(path).unwrap();
        let size = os_file.metadata().map(|m| m.len()).unwrap_or(0);
        let mime_type = mime_guess::from_path(path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        let file = File {
            name,
            path: path.clone(),
            size,
            mime_type,
            parent: Box::new(Node::FolderNode(parent.clone())),
        };

        Ok(file)
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
