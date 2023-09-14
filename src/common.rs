use std::fs::{create_dir_all, File, read_dir};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use toiletcli::colors::{Color, Style};

use crate::docs::Docs;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
// @@@
pub const PROGRAM_NAME: &str = "dedoc";

pub const DEFAULT_DOWNLOADS_LINK: &str = "https://downloads.devdocs.io";
pub const DEFAULT_DOCS_LINK: &str = "https://devdocs.io/docs.json";
pub const DEFAULT_USER_AGENT: &str = "dedoc";

pub const RED: Color = Color::Red;
pub const GREEN: Color = Color::Green;
pub const YELLOW: Color = Color::Yellow;
pub const GRAY: Color = Color::BrightBlack;

pub const BOLD: Style = Style::Bold;
pub const UNDERLINE: Style = Style::Underlined;

pub const RESET: Style = Style::Reset;

#[macro_export]
macro_rules! debug {
    ($($e:expr),+) => {{
            #[cfg(debug_assertions)]
            { dbg!($($e),+) }
            #[cfg(not(debug_assertions))]
            { () }
    }};
}

#[macro_export]
macro_rules! debug_println {
    ($($e:expr),+) => {{
            #[cfg(debug_assertions)]
            { println!($($e),+) }
            #[cfg(not(debug_assertions))]
            { () }
    }};
}

// @@@: idk
pub fn get_home_directory() -> Result<PathBuf, String> {
    fn internal() -> Result<String, String> {
        #[cfg(target_family = "unix")]
        let home = std::env::var("HOME");

        #[cfg(target_family = "windows")]
        let home = std::env::var("userprofile");

        if let Ok(home) = home {
            Ok(home)
        } else {
            let user = std::env::var("USER").map_err(|err| err.to_string())?;

            #[cfg(target_family = "unix")]
            let home = format!("/home/{user}");

            #[cfg(target_family = "windows")]
            let home = format!("C:\\Users\\{user}");

            Ok(home)
        }
    }

    let home = &internal()?;
    let home_path = Path::new(home);

    if home_path.is_dir() {
        Ok(PathBuf::from(home_path))
    } else {
        Err("Could not figure out home directory".to_string())
    }
}

#[inline(always)]
pub fn get_program_directory() -> Result<PathBuf, String> {
    let path = get_home_directory()?;
    let dot_program = format!(".{PROGRAM_NAME}");
    let program_path = path.join(dot_program);
    Ok(program_path)
}

pub fn create_program_directory() -> Result<(), String> {
    let program_path = get_program_directory()?;

    if !program_path.exists() {
        create_dir_all(&program_path)
            .map_err(|err| format!("Could not create {program_path:?}: {err}"))?;
    }

    if program_path.is_dir() {
        Ok(())
    } else {
        Err("Could not create {program_path:?}".to_string())
    }
}

const WEEK: Duration = Duration::from_secs(60 * 60 * 24 * 7);

pub fn is_docs_json_old() -> Result<bool, String> {
    let program_path = get_program_directory().map_err(|err| err.to_string())?;

    let metadata = program_path
        .join("docs.json")
        .metadata()
        .map_err(|err| err.to_string())?;

    let modified_time = metadata.modified().map_err(|err| err.to_string())?;

    let elapsed_time = SystemTime::now()
        .duration_since(modified_time)
        .map_err(|err| err.to_string())?;

    if elapsed_time > WEEK {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn write_to_logfile(message: String) -> Result<PathBuf, String> {
    let log_file_path = get_program_directory()?.join("logs.txt");

    let mut log_file = if log_file_path.exists() {
        File::open(&log_file_path)
    } else {
        File::create(&log_file_path)
    }
    .map_err(|err| format!("Could not open {log_file_path:?}: {err}"))?;

    writeln!(log_file, "{}", message)
        .map_err(|err| format!("Could not write {log_file_path:?}: {err}"))?;

    Ok(log_file_path)
}

#[inline(always)]
pub fn is_docset_in_docs(docset_name: &String, docs: &Vec<Docs>) -> bool {
    let mut found = false;

    for entry in docs.iter() {
        if entry.slug == *docset_name {
            return true;
        }
    }

    false
}

pub fn convert_paths_to_items(paths: Vec<PathBuf>, docset_name: &String) -> Result<Vec<String>, String> {
    let docset_path = get_docset_path(docset_name)?;

    let mut items = vec![];

    for path in paths {
        let item = path
            .strip_prefix(&docset_path)
            .map_err(|err| err.to_string())?;
        let item = item.with_extension("");
        items.push(item.display().to_string());
    }

    Ok(items)
}

pub fn print_search_results(items: Vec<String>, mut start_index: usize) -> Result<(), String> {
    for item in items {
        println!("{GRAY}{start_index:>4}{RESET}  {}", item);
        start_index += 1;
    }

    Ok(())
}

pub fn get_local_docsets() -> Result<Vec<String>, String> {
    let docsets_path = get_program_directory()?.join("docsets");

    let mut docsets_dir = read_dir(docsets_path)
        .map_err(|err| err.to_string())?;

    let mut result = vec![];

    while let Some(entry) = docsets_dir.next() {
        let entry = entry
            .map_err(|err| err.to_string())?;

        let holy_result_option_please_stop = entry.file_name()
            .to_string_lossy()
            .to_string();

        result.push(holy_result_option_please_stop);
    }

    Ok(result)
}

#[inline(always)]
pub fn is_docset_downloaded(docset_name: &String) -> Result<bool, String> {
    get_docset_path(docset_name)?
        .try_exists()
        .map_err(|err| format!("Could not check if `{docset_name}` exists: {err}"))
}

#[inline(always)]
pub fn is_docs_json_exists() -> Result<bool, String> {
    let docs_json_path = get_program_directory()?.join("docs.json");
    Ok(docs_json_path.exists())
}

// so yesterday i deleted a folder i was not meant to
pub fn is_name_allowed<S: ToString>(docset_name: S) -> bool {
    let docset = docset_name.to_string();
    let test_path = PathBuf::from(&docset);

    let has_slashes = {
        #[cfg(target_family = "windows")]
        { docset.find("\\").is_some() || docset.find("/").is_some() }

        #[cfg(target_family = "unix")]
        { docset.find("/").is_some() }
    };
    let starts_with_tilde = docset.starts_with('~');
    let has_dollars = docset.find('$').is_some();
    let is_simple = test_path.canonicalize()
        .map(|path| path == test_path)
        .unwrap_or(true);
    let is_absolute = test_path.is_absolute();

    debug!(has_slashes, has_dollars, is_absolute, is_simple);

    !(has_slashes || starts_with_tilde || has_dollars || is_absolute || !is_simple)
}

#[allow(dead_code)]
pub fn get_zeal_docsets_directory() -> Result<PathBuf, String> {
    let zeal_parent_dir = if cfg!(target_family = "windows") {
        get_home_directory()?
            .join("AppData")
            .join("Local")
    } else {
        get_home_directory()?
            .join(".local")
            .join("share")
    };

    let zeal_docsets_dir = zeal_parent_dir
        .join("Zeal")
        .join("Zeal")
        .join("docsets");

    Ok(zeal_docsets_dir)
}

// win32: %LocalAppData%\Zeal\Zeal\docsets
// unix:  .local/share/Zeal/Zeal/docsets
#[allow(dead_code)]
pub fn is_zeal_installed() -> Result<bool, String> {
    let zeal_docsets_dir = get_zeal_docsets_directory()?;
    let zeal_docsets_exists = zeal_docsets_dir.try_exists()
        .map_err(|err| format!("Could not check if Zeal ({zeal_docsets_dir:?}) exists: {err}"))?;

    Ok(zeal_docsets_exists)
}

#[inline(always)]
pub fn get_docset_path(docset_name: &String) -> Result<PathBuf, String> {
    let docsets_path = get_program_directory()?.join("docsets");
    Ok(docsets_path.join(docset_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_names() {
         let bad_name_path = "/what";
         let bad_name_home = "~";
         let bad_name_dots = "..";
         let bad_name_env  = "$HOME";

         let good_name_simple = "hello";
         let good_name_version = "qt~6.1";
         let good_name_long   = "scala~2.13_reflection";

        assert!(!is_name_allowed(bad_name_path));
        assert!(!is_name_allowed(bad_name_home));
        assert!(!is_name_allowed(bad_name_dots));
        assert!(!is_name_allowed(bad_name_env));

        assert!(is_name_allowed(good_name_simple));
        assert!(is_name_allowed(good_name_version));
        assert!(is_name_allowed(good_name_long));
    }
}
