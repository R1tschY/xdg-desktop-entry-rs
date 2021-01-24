use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::{env, fs};

use nom::Parser;

/// Determine XDG_DATA_DIRS from
/// https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html
pub fn get_data_dirs() -> Vec<PathBuf> {
    let paths = env::var_os("XDG_DATA_DIRS").as_ref().and_then(|dirs| {
        if dirs.is_empty() {
            None
        } else {
            Some(env::split_paths(dirs).collect::<Vec<_>>())
        }
    });

    match paths {
        Some(paths) if !paths.is_empty() => paths,
        _ => vec![
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/usr/share"),
        ],
    }
}

/// Determine XDG_DATA_HOME from
/// https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// Is None when user has no home directory
pub fn get_data_home() -> Option<PathBuf> {
    dirs::data_dir()
}

fn collect_files_recursive(path: &Path) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = vec![];
    collect_files_recursive_(path, &mut result);
    result
}

fn collect_files_recursive_(path: &Path, result: &mut Vec<PathBuf>) {
    if let Ok(dir_entries) = fs::read_dir(path) {
        for dir_entry in dir_entries {
            if let Ok(dir_entry) = dir_entry {
                let path = dir_entry.path();
                if let Ok(file_type) = dir_entry.file_type() {
                    if file_type.is_symlink() {
                        continue;
                    } else if file_type.is_dir() {
                        collect_files_recursive_(&path, result);
                    } else if path.extension() == Some(OsStr::new("desktop")) {
                        result.push(path);
                    }
                }
            }
        }
    }
}

/// Search for application desktop files in default locations.
///
/// Symbolic links are not followed.
///
/// Default locations are:
/// - `applications` subdirectories in `$XDG_DATA_DIRS` or `["/usr/local/share/applications", "/usr/share/applications"]` when `$XDG_DATA_DIRS` is not set
/// - `$XDG_DATA_HOME/applications` or `~/.local/share/applications` when `$XDG_DATA_HOME` is not set
pub fn discover_applications() -> Vec<PathBuf> {
    // TODO: use Desktop File ID to implement precedence order
    get_data_dirs()
        .iter()
        .chain(get_data_home().iter())
        .flat_map(|data_dir| collect_files_recursive(&data_dir.join("applications")))
        .collect()
}

/// Search for desktop files in `dirs`.
///
/// Symbolic links are not followed.
pub fn discover_in_dirs(dirs: &[&Path]) -> Vec<PathBuf> {
    dirs.iter()
        .flat_map(|dir| collect_files_recursive(dir))
        .collect()
}
