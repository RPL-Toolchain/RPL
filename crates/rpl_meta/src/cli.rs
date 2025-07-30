use std::env::current_dir;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::RPLMetaError;

/// Collect default patterns (paths and contents) from the repository.
///
/// The patterns are embedded in the binary, and their paths are *absolute paths*,
/// **as if** the root of the repository is `/rpl/`. Their contents are
/// collected at compile time, and won't change unless re-compiled.
///
/// This is used to provide a set of default patterns that can be used
/// by the user without setting up anything.
pub fn collect_default_patterns() -> Vec<(PathBuf, String)> {
    /// This macro will return a tuple of the path and the content of the file.
    ///
    /// Please pass a path related to `docs/patterns-pest`.
    macro_rules! default_pattern {
        ($path:literal) => {
            (
                PathBuf::from(concat!("/rpl/docs/patterns-pest/", $path)),
                include_str!(concat!("../../../docs/patterns-pest/", $path)).to_owned(),
            )
        };
    }

    macro_rules! default_patterns {
        ($($name:literal),* $(,)?) => {
            vec![$(
                default_pattern!($name),
            )*]
        };
    }

    default_patterns!(
        // Clippy lints
        "clippy/cast-slice-different-sizes.rpl",
        "clippy/cast-slice-from-raw-parts.rpl",
        "clippy/eager-transmute.rpl",
        "clippy/from-raw-with-void-ptr.rpl",
        "clippy/mem-replace-with-uninit.rpl",
        "clippy/mut-from-ref.rpl",
        "clippy/not-unsafe-ptr-arg-deref.rpl",
        "clippy/size-of-in-element-count.rpl",
        "clippy/swap-ptr-to-ref.rpl",
        "clippy/transmute-int-to-non-zero.rpl",
        "clippy/transmute-null-to-fn.rpl",
        "clippy/transmuting-null.rpl",
        "clippy/uninit-assumed-init.rpl",
        "clippy/uninit-vec.rpl",
        "clippy/unsound-collection-transmute.rpl",
        "clippy/wrong-transmute.rpl",
        "clippy/zst-offset.rpl",
        // CVE patterns
        "cve/CVE-2018-20992.rpl",
        "cve/CVE-2018-21000.rpl",
        "cve/CVE-2019-15548.rpl",
        "cve/CVE-2019-16138.rpl",
        "cve/CVE-2020-25016.rpl",
        "cve/CVE-2020-35860.rpl",
        "cve/CVE-2020-35862.rpl",
        "cve/CVE-2020-35873.rpl",
        "cve/CVE-2020-35877.rpl",
        "cve/CVE-2020-35881.rpl",
        "cve/CVE-2020-35887.rpl",
        "cve/CVE-2020-35888.rpl",
        "cve/CVE-2020-35892-3.rpl",
        "cve/CVE-2020-35898-9.rpl",
        "cve/CVE-2020-35901-2.rpl",
        "cve/CVE-2020-35907.rpl",
        "cve/CVE-2021-25904.rpl",
        "cve/CVE-2021-25905.rpl",
        "cve/CVE-2021-27376.rpl",
        "cve/CVE-2021-29941-2.rpl",
        "cve/CVE-2022-23639.rpl",
        "cve/CVE-2024-27284.rpl",
        // Common patterns based on Rust's UB
        "ub/allow-unchecked.rpl",
        "ub/manually-drop.rpl",
        "ub/private-or-generic-function-marked-inline.rpl",
        "ub/transmute-int-to-ptr.rpl",
        "ub/transmute-to-bool.rpl",
    )
}

pub fn collect_file_from_string_args(args: &[String], handler: impl Fn() -> !) -> Vec<(PathBuf, String)> {
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
                            current: current_dir().unwrap_or_else(|_| PathBuf::from("<unknown>")),
                            error: Arc::new(err)
                        }
                    );
                    handler(); // Call the handler to stop execution
                },
            };
            res.push((path, buf));
        });
    }
    res.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));
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
