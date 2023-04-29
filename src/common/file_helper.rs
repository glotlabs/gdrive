use std::io;
use std::fs::File;
use std::path::PathBuf;
use mktemp::Temp;

pub fn stdin_to_file() -> Result<Temp, io::Error> {
    let tmp_file = Temp::new_file()?;
    let path = tmp_file.as_ref().to_path_buf();
    let mut file = File::create(&path)?;
    io::copy(&mut io::stdin(), &mut file)?;
    Ok(tmp_file)
}

pub fn open_file(path: &PathBuf) -> Result<File, io::Error> {
    if PathBuf::from("-") == *path {
        File::open(stdin_to_file()?)
    } else {
        File::open(path)
    }
}
