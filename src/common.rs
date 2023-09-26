use std::fs::{create_dir_all, File, read_dir};
use std::fmt::Display;
use std::sync::Once;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use html2text::from_read_coloured;
use html2text::render::text_renderer::{RichAnnotation, RichAnnotation::*};

use toiletcli::colors::{Color, Style};

use serde::{Deserialize, Serialize};

pub type ResultS = Result<(), String>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROGRAM_NAME: &str = "dedoc";

pub const DEFAULT_DB_JSON_LINK: &str = "https://documents.devdocs.io";
pub const DEFAULT_DOCS_JSON_LINK: &str = "https://devdocs.io/docs.json";

pub const DEFAULT_USER_AGENT: &str = "dedoc";

pub const DOC_PAGE_EXTENSION: &str = "html";

pub const RED:       Color = Color::Red;
pub const GREEN:     Color = Color::Green;
pub const YELLOW:    Color = Color::Yellow;
pub const GRAY:      Color = Color::BrightBlack;
pub const BOLD:      Style = Style::Bold;
pub const UNDERLINE: Style = Style::Underlined;
pub const RESET:     Style = Style::Reset;

#[macro_export]
macro_rules! debug_println {
    ($($e:expr),+) => {{
            #[cfg(debug_assertions)]
            {
                eprint!("{}:{}: ", file!(), line!());
                eprintln!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            { () }
    }};
}

#[inline(always)]
fn unknown_version() -> String {
    "unknown".to_string()
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Links {
    home: String,
    code: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Docs {
    #[serde(skip)]
    name: String,
    pub slug: String,
    #[serde(skip)]
    doctype: String,
    #[serde(skip)]
    links: Links,
    #[serde(default = "unknown_version")]
    pub version: String,
    #[serde(skip)]
    release: String,
    pub mtime: u64,
    db_size: usize,
    #[serde(skip)]
    attribution: String,
}

/*
Example item:
    {
        "name": "Angular",
        "slug": "angular",
        "type": "angular",
        "links": {
          "home": "https://google.com",
          "code": "https://google.com"
        },
        "version": "",
        "release": "16.1.3",
        "mtime": 1688411876,
        "db_size": 13128638,
        "attribution": "whatever"
    }
*/

pub fn deserialize_docs_json() -> Result<Vec<Docs>, String> {
    let docs_json_path = get_program_directory()?.join("docs.json");
    let file = File::open(&docs_json_path)
        .map_err(|err| format!("Could not open `{}`: {err}", docs_json_path.display()))?;

    let reader = BufReader::new(file);

    let docs = serde_json::from_reader(reader)
        .map_err(|err| format!("{err}. Maybe `docs.json` was modified?"))?;

    Ok(docs)
}

fn default_colour_map(annotation: &RichAnnotation) -> (String, String) {
    match annotation {
        Default => ("".into(), "".into()),
        Link(_) => (
            format!("{}", Color::Blue),
            format!("{}", Color::Reset),
        ),
        Image(_) => (
            format!("{}", Color::BrightBlue),
            format!("{}", Color::Reset)
        ),
        Emphasis => (
            format!("{}", Style::Bold),
            format!("{}", Style::Reset),
        ),
        Strong => (
            format!("{}", Style::Bold),
            format!("{}", Style::Reset)
        ),
        Strikeout => (
            format!("{}", Style::Strikethrough),
            format!("{}", Style::Reset)
        ),
        Code => (
            format!("{}", Color::BrightBlack),
            format!("{}", Color::Reset)
        ),
        Preformat(_) => (
            format!("{}", Color::BrightBlack),
            format!("{}", Color::Reset)
        ),
    }
}

pub fn print_docset_file(path: PathBuf, _header: Option<&str>) -> ResultS {
    let file = File::open(&path)
        .map_err(|err| format!("Could not open `{}`: {err}", path.display()))?;
    let reader = BufReader::new(file);

    let page = from_read_coloured(reader, 80, default_colour_map)
        .map_err(|err| err.to_string())?;

    println!("{}", page.trim());

    Ok(())
}

pub fn print_page_from_docset(docset_name: &String, page: &String) -> ResultS {
    let docset_path = get_docset_path(docset_name)?;

    let mut page_split = page.split('#');

    let page_path = if let Some(file) = page_split.next() {
        Ok(file)
    } else {
        Err(format!("Invalid page: {page}"))
    }?;

    let header = page_split.next();

    let page_path_string = docset_path.join(page_path)
        .display()
        .to_string() + "." + DOC_PAGE_EXTENSION;
    let page_path = PathBuf::from(page_path_string);

    if !page_path.is_file() {
        let message = format!(
            "\
No page matching `{page}`. Did you specify the name from `search` correctly?"
        );
        return Err(message);
    }

    print_docset_file(page_path, header)
}

static mut HOME_DIR: Option<PathBuf> = None;
static HOME_DIR_INIT: Once = Once::new();

pub fn get_home_directory() -> Result<PathBuf, String> {
    unsafe {
        if let Some(home_dir) = HOME_DIR.as_ref() {
            return Ok(home_dir.clone());
        }
    }

    fn internal() -> Result<PathBuf, String> {
        #[cfg(target_family = "unix")]
        let home = std::env::var("HOME");

        #[cfg(target_family = "windows")]
        let home = std::env::var("userprofile");

        if let Ok(home) = home {
            Ok(home.into())
        } else {
            let user = std::env::var("USER").map_err(|err| err.to_string())?;

            #[cfg(target_family = "unix")]
            let home = format!("/home/{user}");

            #[cfg(target_family = "windows")]
            let home = format!("C:\\Users\\{user}");

            Ok(home.into())
        }
    }

    let home_path = internal()?;

    if home_path.is_dir() {
        unsafe {
            HOME_DIR_INIT.call_once(|| {
                HOME_DIR = Some(home_path.clone());
            });
        }
        Ok(home_path)
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

pub fn create_program_directory() -> ResultS {
    let program_path = get_program_directory()?;

    if !program_path.exists() {
        create_dir_all(&program_path)
            .map_err(|err| format!("Could not create `{}`: {err}", program_path.display()))?;
    }

    if program_path.is_dir() {
        Ok(())
    } else {
        Err("Could not create {program_path:?}".to_string())
    }
}

const WEEK: Duration = Duration::from_secs(60 * 60 * 24 * 7);

pub fn is_docs_json_old() -> Result<bool, String> {
    let program_path = get_program_directory()
        .map_err(|err| err.to_string())?;

    let metadata = program_path
        .join("docs.json")
        .metadata()
        .map_err(|err| err.to_string())?;

    let modified_time = metadata.modified()
        .map_err(|err| err.to_string())?;

    let elapsed_time = SystemTime::now()
        .duration_since(modified_time)
        .map_err(|err| err.to_string())?;

    if elapsed_time > WEEK {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn write_to_logfile(message: impl Display) -> Result<PathBuf, String> {
    let log_file_path = get_program_directory()?.join("logs.txt");

    let mut log_file = if log_file_path.exists() {
        File::options().append(true).open(&log_file_path)
    } else {
        File::create(&log_file_path)
    }
    .map_err(|err| format!("Could not open `{}`: {err}", log_file_path.display()))?;

    writeln!(log_file, "{}", message)
        .map_err(|err| format!("Could not write `{}`: {err}", log_file_path.display()))?;

    Ok(log_file_path)
}

pub enum SearchMatch {
    Found,
    FoundVague(Vec<String>)
}

pub fn is_docset_in_docs_or_print_warning(docset_name: &String, docs: &Vec<Docs>) -> bool {
    match is_docset_in_docs(docset_name, docs) {
        Some(SearchMatch::Found) => return true,
        Some(SearchMatch::FoundVague(vague_matches)) => {
            let first_three = &vague_matches[..3];
            println!("{YELLOW}WARNING{RESET}: Unknown docset `{docset_name}`. Did you mean `{}`?", first_three.join("`/`"));
        }
        None => {
            println!("{YELLOW}WARNING{RESET}: Unknown docset `{docset_name}`. Did you run `fetch`?");
        }
    }
    false
}

pub fn is_docset_in_docs(docset_name: &String, docs: &Vec<Docs>) -> Option<SearchMatch> {
    let mut vague_matches = vec![];

    for entry in docs.iter() {
        if entry.slug.contains(docset_name) {
            if entry.slug == *docset_name {
                return Some(SearchMatch::Found);
            }
            vague_matches.push(entry.slug.clone());
        }
    }

    if vague_matches.is_empty() {
        None
    } else {
        Some(SearchMatch::FoundVague(vague_matches))
    }
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

pub fn print_search_results(search_results: &[String], mut start_index: usize) -> ResultS {
    for item in search_results {
        if let Some(header_index) = item.rfind('#') {
            println!("{GRAY}{start_index:>4}{RESET}  {}{GRAY}, #{}", &item[..header_index], &item[header_index + 1..]);
        } else {
            println!("{GRAY}{start_index:>4}{RESET}  {}", item);
        }
        start_index += 1;
    }
    Ok(())
}

pub fn get_local_docsets() -> Result<Vec<String>, String> {
    let docsets_path = get_program_directory()?.join("docsets");
    let docsets_dir_exists = docsets_path.try_exists()
        .map_err(|err| format!("Could not check `{}`: {err}", docsets_path.display()))?;

    let mut result = vec![];

    // `/docsets` does not exist, return empty vector
    if !docsets_dir_exists {
        return Ok(result);
    }

    let mut docsets_dir = read_dir(docsets_path)
        .map_err(|err| err.to_string())?;

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

pub fn is_name_allowed<S: AsRef<str>>(docset_name: &S) -> bool {
    let docset = docset_name.as_ref();

    let has_slashes = {
        #[cfg(target_family = "windows")]
        { docset.find("\\").is_some() || docset.find("/").is_some() }

        #[cfg(target_family = "unix")]
        { docset.find("/").is_some() }
    };
    let starts_with_tilde = docset.starts_with('~');
    let has_dollars = docset.find('$').is_some();
    let starts_with_dot = docset.starts_with('.');
    let has_dots = docset.find("..").is_some();

    !(has_slashes || starts_with_tilde || has_dollars || starts_with_dot || has_dots)
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

         let good_name_simple  = "hello";
         let good_name_version = "qt~6.1";
         let good_name_long    = "scala~2.13_reflection";

        assert!(!is_name_allowed(&bad_name_path));
        assert!(!is_name_allowed(&bad_name_home));
        assert!(!is_name_allowed(&bad_name_dots));
        assert!(!is_name_allowed(&bad_name_env));

        assert!(is_name_allowed(&good_name_simple));
        assert!(is_name_allowed(&good_name_version));
        assert!(is_name_allowed(&good_name_long));
    }
}
