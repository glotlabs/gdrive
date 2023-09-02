use mktemp::Temp;
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub fn stdin_to_file() -> Result<Temp, io::Error> {
    let tmp_file = Temp::new_file()?;
    let path = tmp_file.as_ref().to_path_buf();
    let mut file = File::create(&path)?;
    io::copy(&mut io::stdin(), &mut file)?;
    Ok(tmp_file)
}

pub fn open_file(path: &Option<PathBuf>) -> Result<(File, PathBuf), io::Error> {
    match path {
        Some(path) => {
            let file = File::open(path)?;
            Ok((file, path.clone()))
        }
        None => {
            let tmp_file = stdin_to_file()?;
            let path = tmp_file.as_ref().to_path_buf();
            let file = File::open(&path)?;
            Ok((file, path))
        }
    }
}
