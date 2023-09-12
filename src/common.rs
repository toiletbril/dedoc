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
            found = true;
        }
    }

    found
}

#[inline(always)]
pub fn print_search_results(paths: Vec<PathBuf>, docset_name: &String) -> Result<(), String> {
    let docset_path = get_docset_path(docset_name)?;

    for path in paths {
        let item = path
            .strip_prefix(&docset_path)
            .map_err(|err| err.to_string())?;
        let item = item.with_extension("");
        println!("  {}", item.display());
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

#[inline(always)]
pub fn get_docset_path(docset_name: &String) -> Result<PathBuf, String> {
    let docsets_path = get_program_directory()?.join("docsets");
    if PathBuf::from(docset_name).is_absolute() {
        return Err("Absolute paths are not allowed".to_string());
    }
    Ok(docsets_path.join(docset_name))
}
