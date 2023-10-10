use std::fs::{create_dir_all, File, read_dir};
use std::fmt::Display;
use std::sync::Once;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use html2text::render::text_renderer::{RichAnnotation, TaggedLine, TaggedString, TaggedLineElement::*};

use toiletcli::colors::{Color, Style};

use serde::{Deserialize, Serialize};

pub(crate) type ResultS = Result<(), String>;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
pub(crate) const PROGRAM_NAME: &str = "dedoc_debug";
#[cfg(not(debug_assertions))]
pub(crate) const PROGRAM_NAME: &str = "dedoc";

pub(crate) const DEFAULT_DB_JSON_LINK: &str   = "https://documents.devdocs.io";
pub(crate) const DEFAULT_DOCS_JSON_LINK: &str = "https://devdocs.io/docs.json";

pub(crate) const DEFAULT_USER_AGENT: &str = "dedoc";

pub(crate) const DOC_PAGE_EXTENSION: &str = "html";

pub(crate) const RED:        Color = Color::Red;
pub(crate) const GREEN:      Color = Color::Green;
pub(crate) const YELLOW:     Color = Color::Yellow;
pub(crate) const LIGHT_GRAY: Color = Color::Byte(248);
pub(crate) const GRAY:       Color = Color::BrightBlack;
pub(crate) const GRAYER:     Color = Color::Byte(240);
pub(crate) const GRAYEST:    Color = Color::Byte(234);
pub(crate) const BOLD:       Style = Style::Bold;
pub(crate) const UNDERLINE:  Style = Style::Underlined;
pub(crate) const RESET:      Style = Style::Reset;

#[macro_export]
macro_rules! debug_println {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        {
            eprint!("{}:{}: {RED}", file!(), line!());
            eprintln!($($e),+);
            eprint!("{RESET}");
        }
        #[cfg(not(debug_assertions))]
        { () }
    };
}

#[macro_export]
macro_rules! dedoc_dbg {
    ($($e:expr),+) => {
        #[cfg(debug_assertions)]
        {
            eprint!("{RED}", file!(), line!());
            dbg!($($e),+);
        }
        #[cfg(not(debug_assertions))]
        { () }
    };
}

#[inline]
fn unknown_version() -> String {
    "unknown".to_string()
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[derive(Default)]
pub(crate) struct Links {
    home: String,
    code: String,
}

// docs.json
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub(crate) struct Docs {
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

// Example entry:
// {
//     "name": "Angular",
//     "slug": "angular",
//     "type": "angular",
//     "links": {
//       "home": "https://google.com",
//       "code": "https://google.com"
//     },
//     "version": "",
//     "release": "16.1.3",
//     "mtime": 1688411876,
//     "db_size": 13128638,
//     "attribution": "whatever"
// }

pub(crate) fn deserialize_docs_json() -> Result<Vec<Docs>, String> {
    let docs_json_path = get_program_directory()?.join("docs.json");
    let file = File::open(&docs_json_path)
        .map_err(|err| format!("Could not open `{}`: {err}", docs_json_path.display()))?;

    let reader = BufReader::new(file);

    let docs = serde_json::from_reader(reader)
        .map_err(|err| format!("{err}. Maybe `docs.json` was modified?"))?;

    Ok(docs)
}

#[macro_export]
macro_rules! print_warning {
    ($($e:expr),+) => {
        {
            eprint!("{}WARNING{}: ", toiletcli::colors::Color::Yellow, toiletcli::colors::Style::Reset);
            eprintln!($($e),+);
        }
    };
}

#[inline]
pub(crate) fn split_to_item_and_fragment(path: String) -> Result<(String, Option<String>), String> {
    let mut path_split = path.split('#');

    let item = if let Some(item) = path_split.next() {
        Ok(item)
    } else {
        Err(format!("Invalid page path: {}", path))
    }?.to_owned();

    let fragment = path_split.next()
        .map(|s| s.to_owned());

    Ok((item, fragment))
}

fn get_tag_style(tagged_string_tags: &Vec<RichAnnotation>) -> String {
    let mut style = String::new();
    let mut temp_style;

    for annotation in tagged_string_tags {
        temp_style = match *annotation {
            RichAnnotation::Default => continue,
            RichAnnotation::Link(_) => {
                format!("{}", Color::Blue)
            }
            RichAnnotation::Image(_) => {
                format!("{}", Color::BrightBlue)
            }
            RichAnnotation::Emphasis => {
                format!("{}", Style::Bold)
            }
            RichAnnotation::Strong => {
                format!("{}", Style::Bold)
            }
            RichAnnotation::Strikeout => {
                format!("{}", Style::Strikethrough)
            },
            RichAnnotation::Code => {
                format!("{}", Color::BrightBlack)
            }
            RichAnnotation::Preformat(_) => {
                format!("{}{}", LIGHT_GRAY, GRAYEST.bg())
            }
        };

        style.push_str(&temp_style)
    }

    style
}

// This function ignores fragment's character case, to support --case-insensitive
fn get_fragment_bounds(
    tagged_lines: &[TaggedLine<Vec<RichAnnotation>>],
    fragment: &str
) -> (Option<usize>, Option<usize>)
{
    let lowercase_fragment = fragment.to_lowercase();

    let mut current_fragment_line = None;
    let mut found_fragment = false;

    for (line_number, tagged_line) in tagged_lines.iter().enumerate() {
        for tagged_line_element in tagged_line.iter() {
            match tagged_line_element {
                FragmentStart(temp_fragment) if temp_fragment.to_lowercase() == lowercase_fragment => {
                    current_fragment_line = Some(line_number);
                    found_fragment = true;
                }
                FragmentStart(_) if found_fragment => {
                    let next_fragment_line = Some(line_number);
                    return (current_fragment_line, next_fragment_line);
                }
                _ => {}
            }
        }
    }

    (current_fragment_line, None)
}

pub(crate) fn print_docset_file(path: PathBuf, fragment: Option<&String>) -> Result<bool, String> {
    let file = File::open(&path)
        .map_err(|err| format!("Could not open `{}`: {err}", path.display()))?;
    let reader = BufReader::new(file);

    let rich_page = html2text::from_read_rich(reader, 80);

    let mut current_fragment_line = 0;
    let mut next_fragment_line = 0;

    let mut is_fragment_found = false;
    let mut has_next_fragment = false;

    // If there is a fragment, determine current fragment offset and print
    // everything until the next fragment.
    if let Some(fragment) = fragment {
        let (current_fragment, next_fragment) = get_fragment_bounds(&rich_page, fragment);

        if let Some(line) = current_fragment {
            current_fragment_line = line;
            is_fragment_found = true;
        }

        if let Some(line) = next_fragment {
            next_fragment_line = line;
            has_next_fragment = true;
        }

        // @@@: figure out better way to short-circuit search when it fails a test
        #[cfg(debug_assertions)]
        if !is_fragment_found {
            return Err(format!("debug: #{fragment} is specified but wasn't found in the page"));
        }
    }

    if is_fragment_found {
        println!("{GRAYER}...{RESET}")
    }

    let mut skipped_empty_lines = false;

    for (i, rich_line) in rich_page.iter().enumerate() {
        if is_fragment_found && i < current_fragment_line  { continue; }
        if has_next_fragment && i > next_fragment_line { break; }

        let tagged_strings: Vec<&TaggedString<Vec<RichAnnotation>>> = rich_line
            .tagged_strings()
            .collect();

        let mut line_is_empty = true;
        let is_only_tag = tagged_strings.len() == 1;

        let mut line_buffer = String::new();

        for tagged_string in tagged_strings {
            let style = get_tag_style(&tagged_string.tag);

            if !tagged_string.s.is_empty() {
                line_is_empty = false;
            }

            line_buffer += style.as_str();
            line_buffer += &tagged_string.s;

            if is_only_tag {
                // Pad preformat to 80 characters for cool background.
                if let Some(RichAnnotation::Preformat(_)) = tagged_string.tag.first() {
                    let padding_amount = 80_usize
                        .saturating_sub(tagged_string.s.len());

                    for _ in 0..padding_amount {
                        line_buffer += " ";
                    }
                }
            }

            line_buffer += &Style::Reset.to_string();
        }

        if !line_is_empty {
            skipped_empty_lines = true;
        }

        if skipped_empty_lines {
            println!("{}", line_buffer);
        }

        line_buffer.clear();
    }

    if has_next_fragment {
        println!("{GRAYER}...{RESET}")
    }

    Ok(is_fragment_found)
}

pub(crate) fn print_page_from_docset(docset_name: &str, page: &str, fragment: Option<&String>) -> Result<bool, String> {
    let docset_path = get_docset_path(docset_name)?;

    let page_path_string = docset_path.join(page)
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

    print_docset_file(page_path, fragment)
}

static mut HOME_DIR: Option<PathBuf> = None;
static HOME_DIR_INIT: Once = Once::new();

pub(crate) fn get_home_directory() -> Result<PathBuf, String> {
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

#[inline]
pub(crate) fn get_program_directory() -> Result<PathBuf, String> {
    let path = get_home_directory()?;
    let dot_program = format!(".{PROGRAM_NAME}");
    let program_path = path.join(dot_program);
    Ok(program_path)
}

pub(crate) fn create_program_directory() -> ResultS {
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

pub(crate) fn is_docs_json_old() -> Result<bool, String> {
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

pub(crate) fn write_to_logfile(message: impl Display) -> Result<PathBuf, String> {
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

pub(crate) enum SearchMatch {
    Exact,
    Vague(Vec<String>),
    None
}

// Returns `true` when docset exists in `docs.json`, print a warning otherwise.
pub(crate) fn is_docset_in_docs_or_print_warning(docset_name: &String, docs: &[Docs]) -> bool {
    match is_docset_in_docs(docset_name, docs) {
        SearchMatch::Exact => return true,
        SearchMatch::Vague(vague_matches) => {
            let end_index = std::cmp::min(3, vague_matches.len());
            let first_three = &vague_matches[..end_index];

            print_warning!("Unknown docset `{docset_name}`. Did you mean `{}`?", first_three.join("`/`"));
        }
        SearchMatch::None => {
            print_warning!("Unknown docset `{docset_name}`. Did you run `fetch`?");
        }
    }
    false
}

// `exact` is a perfect match, `vague` are files that contain `docset_name` in their path.
pub(crate) fn is_docset_in_docs(docset_name: &String, docs: &[Docs]) -> SearchMatch {
    let mut vague_matches = vec![];

    for entry in docs.iter() {
        if entry.slug.contains(docset_name) {
            if entry.slug == *docset_name {
                return SearchMatch::Exact;
            }
            vague_matches.push(entry.slug.clone());
        }
    }

    if vague_matches.is_empty() {
        SearchMatch::None
    } else {
        SearchMatch::Vague(vague_matches)
    }
}

pub(crate) fn get_local_docsets() -> Result<Vec<String>, String> {
    let docsets_path = get_program_directory()?.join("docsets");
    let docsets_dir_exists = docsets_path.try_exists()
        .map_err(|err| format!("Could not check `{}`: {err}", docsets_path.display()))?;

    let mut result = vec![];

    if !docsets_dir_exists {
        return Ok(result);
    }

    let docsets_dir = read_dir(docsets_path)
        .map_err(|err| err.to_string())?;

    for entry in docsets_dir {
        let entry = entry
            .map_err(|err| err.to_string())?;

        let holy_result_option_please_stop = entry.file_name()
            .to_string_lossy()
            .to_string();

        result.push(holy_result_option_please_stop);
    }

    Ok(result)
}

#[inline]
pub(crate) fn is_docset_downloaded(docset_name: &String) -> Result<bool, String> {
    get_docset_path(docset_name)?
        .try_exists()
        .map_err(|err| format!("Could not check if `{docset_name}` exists: {err}"))
}

#[inline]
pub(crate) fn is_docs_json_exists() -> Result<bool, String> {
    let docs_json_path = get_program_directory()?.join("docs.json");
    Ok(docs_json_path.exists())
}

#[inline]
pub(crate) fn get_docset_path(docset_name: &str) -> Result<PathBuf, String> {
    let docsets_path = get_program_directory()?.join("docsets");
    Ok(docsets_path.join(docset_name))
}
