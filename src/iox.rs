use anyhow::{Result, ensure};
use std::fs::{DirEntry, File};
use std::io::{self, BufRead};
use std::path::Path;

pub fn read_lines(file: Option<&Path>) -> Result<Box<dyn Iterator<Item = String>>> {
    if let Some(p) = file {
        let f = File::open(p)?;
        let f = io::BufReader::new(f);
        let it = f.lines().map_while(Result::ok);
        Ok(Box::new(it))
    } else {
        let stdin = io::stdin();
        let it = stdin.lock().lines().map_while(Result::ok);
        Ok(Box::new(it))
    }
}

pub fn read_dirs(root: &Path) -> Result<Box<dyn Iterator<Item = DirEntry>>> {
    ensure!(
        root.is_dir(),
        "cannot read dir because root is not a directory"
    );
    ensure!(root.exists(), "cannot read dir because root is not exist");
    let it = root.read_dir()?;
    let it = it.map_while(Result::ok).filter(|x| x.path().is_dir());
    Ok(Box::new(it))
}
