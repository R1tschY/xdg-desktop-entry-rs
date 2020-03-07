use std::path::PathBuf;
use std::{env, fs};

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
        _ => vec![PathBuf::from("/usr/local/share"), PathBuf::from("/usr/share")]
    }
}

/// Determine XDG_DATA_HOME from
/// https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// Is None when user has no home directory
pub fn get_data_home() -> Option<PathBuf> {
    dirs::data_dir()
}


fn collect_files_recursive<T>(path: &PathBuf, filter: T) -> Vec<PathBuf>
    where T: Fn(&PathBuf) -> bool
{
    let mut result: Vec<PathBuf> = vec![];
    collect_files_recursive_(path, &mut result, &filter);
    result
}

fn collect_files_recursive_<T>(path: &PathBuf, result: &mut Vec<PathBuf>, filter: &T)
    where T: Fn(&PathBuf) -> bool
{
    if let Ok(dir_entries) = fs::read_dir(path) {
        for dir_entry in dir_entries {
            if let Ok(dir_entry) = dir_entry {
                let path = dir_entry.path();
                if path.is_dir() {
                    collect_files_recursive_(&path, result, filter);
                } else if filter(&path) {
                    result.push(path);
                }
            }
        }
    }
}

pub fn get_desktop_files() -> Vec<PathBuf> {
    let desktop_ext: PathBuf = ".desktop".into();

    get_data_dirs().iter()
        .chain(get_data_home().iter())
        .flat_map(|data_dir| {
            collect_files_recursive(data_dir, |p| p.ends_with(&desktop_ext))
        })
        .collect()
}
