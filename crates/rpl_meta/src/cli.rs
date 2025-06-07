use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::RPLMetaError;

pub fn collect_file_from_string_args(args: &[String]) -> Vec<(PathBuf, String)> {
    let mut res = vec![];
    for arg in args {
        traverse_rpl(arg.into(), |path| {
            let buf = read_file_from_path_buf(&path);
            let buf = match buf {
                Ok(buf) => buf,
                Err(err) => {
                    warn!(
                        "{}",
                        RPLMetaError::FileError {
                            path,
                            error: Arc::new(err)
                        }
                    );
                    return;
                },
            };
            res.push((path, buf));
        });
    }
    res
}

fn is_rpl(path: &OsStr) -> bool {
    // debug_eprintln!("Checking if is .rpl: {:?}", path);
    PathBuf::from(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "rpl" || ext == "RPL")
        .unwrap_or(false)
}

fn is_hidden(path: &OsStr) -> bool {
    // debug_eprintln!("Checking if is hidden: {:?}", path);
    path.to_str().map(|name| name.starts_with('.')).unwrap_or(true)
}

/// Read file from path buffer to string.
pub fn read_file_from_path_buf(file: impl AsRef<Path> + Debug) -> io::Result<String> {
    // eprintln!("Reading {:?}", file);
    let content = { std::fs::read_to_string(file)? };
    Ok(content)
}

fn read_dir(dir: &PathBuf) -> Option<impl Iterator<Item = io::Result<(PathBuf, OsString)>>> {
    std::fs::read_dir(dir)
        .map(|dir| dir.map(|dir| dir.map(|dir| (dir.path(), dir.file_name()))))
        .ok()
}

fn traverse_dir(
    stack: &mut Vec<(PathBuf, OsString)>,
    dir: impl Iterator<Item = io::Result<(PathBuf, OsString)>>,
    path: &PathBuf,
) {
    debug!("Walking into {:?}", path);
    for entry in dir {
        match entry {
            Ok(entry) => stack.push(entry),
            Err(err) => warn!("Can't read entry under {:?} because of:\n{}", path, err),
        }
    }
}

/// Traverse all `.rpl` files under a repository.
pub fn traverse_rpl(root: PathBuf, mut f: impl FnMut(PathBuf)) {
    let mut stack: Vec<(PathBuf, OsString)> = vec![];

    if let Some(dir) = read_dir(&root) {
        traverse_dir(&mut stack, dir, &root);
    } else {
        // debug_eprintln!("Running {:?} because it's not a directory.", root);
        f(root);
    }

    while let Some(next) = stack.pop() {
        let (full, file) = next;
        if !is_hidden(&file) {
            if let Some(dir) = read_dir(&full) {
                traverse_dir(&mut stack, dir, &full);
            } else if is_rpl(&file) {
                let res = std::fs::canonicalize(&full);
                match res {
                    Ok(full) => {
                        debug!("{:?} is a normal file which ends with `.rpl`.", full,);
                        f(full);
                    },
                    Err(err) => warn!("Can't canonicalize {:?} because of:\n{}", full, err),
                }
            } else {
                debug!("Skipped {:?}.", full);
            }
        }
    }
}
