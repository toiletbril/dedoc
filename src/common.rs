#![allow(dead_code)]

use std::fmt::Display;
use std::fs::{create_dir_all, read_dir, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::{Duration, SystemTime};

use html2text::render::text_renderer::{
  RichAnnotation, TaggedLine, TaggedLineElement::*, TaggedString,
};
use html2text::Colour;

use toiletcli::colors::{Color, Style};
use toiletcli::flags::{FlagError, FlagErrorType};

use serde::{Deserialize, Serialize};

#[cfg(debug_assertions)]
pub(crate) const PROGRAM_NAME: &str = "dedoc_debug";
#[cfg(not(debug_assertions))]
pub(crate) const PROGRAM_NAME: &str = "dedoc";

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const DEFAULT_DB_JSON_LINK: &str = "https://documents.devdocs.io";
pub(crate) const DEFAULT_DOCS_JSON_LINK: &str = "https://devdocs.io/docs.json";
pub(crate) const DEFAULT_USER_AGENT: &str = "dedoc";
pub(crate) const DEFAULT_PROGRAM_DIR_ENV_VARIABLE: &str = "DEDOC_HOME";
pub(crate) const DEFAULT_WIDTH: usize = 80;

pub(crate) const MTIME_FILENAME: &str = ".dedoc_mtime";
pub(crate) const DOC_PAGE_EXTENSION: &str = "html";

pub(crate) const RED: Color = Color::Red;
pub(crate) const CYAN: Color = Color::Cyan;
pub(crate) const GREEN: Color = Color::Green;
pub(crate) const YELLOW: Color = Color::Yellow;
pub(crate) const LIGHT_GRAY: Color = Color::Byte(248);
pub(crate) const GRAY: Color = Color::BrightBlack;
pub(crate) const GRAYER: Color = Color::Byte(240);
pub(crate) const GRAYEST: Color = Color::Byte(234);
pub(crate) const BOLD: Style = Style::Bold;
pub(crate) const UNDERLINE: Style = Style::Underlined;
pub(crate) const RESET: Style = Style::Reset;

pub(crate) type ResultS = Result<(), String>;

#[macro_export]
macro_rules! debug_println
{
  ($($e:expr),+) =>
  {
    #[cfg(debug_assertions)]
    {
      eprint!("{}:{}: {}", file!(), line!(), toiletcli::colors::Color::Cyan);
      eprintln!($($e),+);
      eprint!("{RESET}");
    }
    #[cfg(not(debug_assertions))]
    { () }
  };
}

#[macro_export]
macro_rules! dedoc_dbg
{
  ($($e:expr),+) =>
  {
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
fn unknown_version() -> String
{
  "unknown".to_string()
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Links
{
  home: String,
  code: String,
}

// Entries from docs.json
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub(crate) struct DocsEntry
{
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

pub(crate) fn deserialize_docs_json() -> Result<Vec<DocsEntry>, String>
{
  let docs_json_path = get_program_directory()?.join("docs.json");
  let file = File::open(&docs_json_path).map_err(|err| {
                                          format!("Could not open `{}`: {err}",
                                                  docs_json_path.display())
                                        })?;

  let reader = BufReader::new(file);

  let docs = serde_json::from_reader(reader)
    .map_err(|err| format!("{err}. Maybe `docs.json` was modified?"))?;

  Ok(docs)
}

#[macro_export]
macro_rules! print_warning
{
  ($($e:expr),+) =>
  {
    {
      eprint!("{}WARNING{}: ", toiletcli::colors::Color::Yellow,
              toiletcli::colors::Style::Reset);
      eprintln!($($e),+);
    }
  };
}

pub(crate) fn get_flag_error(flag_error: &FlagError) -> String
{
  match flag_error.error_type {
    FlagErrorType::CannotCombine => {
      format!("Flag `{}` cannot be combined", flag_error.flag)
    }
    FlagErrorType::NoValueProvided => {
      format!("No value provided for `{}` flag", flag_error.flag)
    }
    FlagErrorType::ExtraValueProvided => {
      format!("Flag `{}` does not take a value", flag_error.flag)
    }
    FlagErrorType::Unknown => {
      format!("Unknown flag `{}`", flag_error.flag)
    }
  }
}

pub(crate) fn get_terminal_width() -> usize
{
  if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size() {
    if w < 120 {
      return w as usize;
    }
    return 120;
  }
  DEFAULT_WIDTH
}

#[inline]
pub(crate) fn split_to_item_and_fragment(
  path: String)
  -> Result<(String, Option<String>), String>
{
  let mut path_split = path.split('#');

  let item = if let Some(item) = path_split.next() {
               Ok(item)
             } else {
               Err(format!("Invalid page path: {}", path))
             }?.to_owned();

  let fragment = path_split.next().map(|s| s.to_owned());

  Ok((item, fragment))
}

fn get_tag_style(tagged_string_tags: &Vec<RichAnnotation>) -> String
{
  let mut style_buffer = String::new();
  let mut temp_style;

  for annotation in tagged_string_tags {
    temp_style = match *annotation {
      RichAnnotation::Default => continue,
      RichAnnotation::Link(_) => format!("{}", Color::Blue),
      RichAnnotation::Image(_) => format!("{}", Color::BrightBlue),
      RichAnnotation::Emphasis => format!("{}", Style::Bold),
      RichAnnotation::Strong => format!("{}", Style::Bold),
      RichAnnotation::Strikeout => format!("{}", Style::Strikethrough),
      RichAnnotation::Code => format!("{}", Color::BrightBlack),
      RichAnnotation::Preformat(_) => format!("{}{}", LIGHT_GRAY, GRAYEST.bg()),

      RichAnnotation::Colour(Colour { r, g, b }) => {
        format!("{}", Color::RGB(r, g, b))
      }
      RichAnnotation::BgColour(Colour { r, g, b }) => {
        Color::RGB(r, g, b).to_string()
      }
      _ => continue,
    };

    style_buffer.push_str(&temp_style)
  }

  style_buffer
}

// This function ignores fragment's character case, to support
// --case-insensitive
fn get_fragment_bounds(tagged_lines: &[TaggedLine<Vec<RichAnnotation>>],
                       fragment: &str)
                       -> (Option<usize>, Option<usize>)
{
  let lowercase_fragment = fragment.to_lowercase();

  let mut current_fragment_line = None;
  let mut found_fragment = false;

  for (line_number, tagged_line) in tagged_lines.iter().enumerate() {
    for tagged_line_element in tagged_line.iter() {
      match tagged_line_element {
        FragmentStart(temp_fragment)
          if temp_fragment.to_lowercase() == lowercase_fragment =>
        {
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

pub(crate) fn print_docset_file(path: PathBuf,
                                fragment: Option<&String>,
                                width: usize,
                                number_lines: bool)
                                -> Result<bool, String>
{
  let file = File::open(&path).map_err(|err| {
                                format!("Could not open `{}`: {err}",
                                        path.display())
                              })?;
  let reader = BufReader::new(file);

  // If we are outputting line numbers, leave 7 columns for ourselves.
  let actual_width = if number_lines { width - 7 } else { width };

  let rich_page = html2text::from_read_rich(reader, actual_width);

  let mut current_fragment_line = 0;
  let mut next_fragment_line = 0;

  let mut is_fragment_found = false;
  let mut has_next_fragment = false;

  // If there is a fragment, determine current fragment offset and print
  // everything until the next fragment.
  if let Some(fragment) = fragment {
    let (current_fragment, next_fragment) =
      get_fragment_bounds(&rich_page, fragment);

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
      return Err(format!(
        "debug: #{fragment} is specified but wasn't found in the page"
      ));
    }
  }

  if is_fragment_found {
    println!("{GRAYER}...{RESET}")
  }

  let mut skipped_empty_lines = false;
  let mut line_number = 0;

  for (i, rich_line) in rich_page.iter().enumerate() {
    if is_fragment_found && i < current_fragment_line {
      continue;
    }
    if has_next_fragment && i > next_fragment_line {
      break;
    }

    let tagged_strings: Vec<&TaggedString<Vec<RichAnnotation>>> =
      rich_line.tagged_strings().collect();

    let is_only_tag = tagged_strings.len() == 1;

    let mut line_is_empty = true;
    let mut line_buffer = String::new();

    if number_lines {
      line_number += 1;
      line_buffer += &format!("{GRAYER}{line_number:>5}{RESET}  ");
    }

    for tagged_string in tagged_strings {
      let style = get_tag_style(&tagged_string.tag);

      if !tagged_string.s.is_empty() {
        line_is_empty = false;
      }

      line_buffer += style.as_str();
      line_buffer += &tagged_string.s;

      if is_only_tag {
        // Pad preformat to terminal width for cool background.
        if let Some(RichAnnotation::Preformat(_)) = tagged_string.tag.first() {
          let padding_amount =
            actual_width.saturating_sub(tagged_string.s.len());

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

pub(crate) fn print_page_from_docset(docset_name: &str,
                                     page: &str,
                                     fragment: Option<&String>,
                                     width: usize,
                                     number_lines: bool)
                                     -> Result<bool, String>
{
  let docset_path = get_docset_path(docset_name)?;

  let page_path_string =
    docset_path.join(page).display().to_string() + "." + DOC_PAGE_EXTENSION;
  let page_path = PathBuf::from(page_path_string);

  if !page_path.is_file() {
    return Err(format!("No page matching `{page}`. Did you specify the name \
                        from `search` correctly?"));
  }

  print_docset_file(page_path, fragment, width, number_lines)
}

fn get_home_directory() -> Result<PathBuf, String>
{
  #[cfg(target_family = "unix")]
  let home_env = std::env::var("HOME");
  #[cfg(target_family = "windows")]
  let home_env = std::env::var("userprofile");

  let home: PathBuf = if let Ok(home_path) = home_env {
    PathBuf::from(home_path)
  } else if cfg!(target_family = "unix") {
    let user =
      std::env::var("USER").map_err(|err| {
                             format!("Could not get $USER variable: {err}")
                           })?;
    format!("/home/{user}").into()
  } else if cfg!(target_family = "windows") {
    let user = std::env::var("USERNAME").map_err(|err| {
                 format!("Could not get $USERNAME variable: {err}")
               })?;
    format!("C:\\Users\\{user}").into()
  } else {
    unreachable!();
  };

  match home.try_exists() {
    Ok(true) => Ok(home),
    Ok(false) => {
      Err(format!("Your home directory (`{}`) does not exist. This may be \
                   caused due to user name and home folder's name mismatch. \
                   Making a symlink may help.",
                  home.display()))
    }
    Err(err) => Err(format!("Could not figure out home directory: {err}")),
  }
}

static mut PROGRAM_DIRECTORY: Option<PathBuf> = None;
static PROGRAM_DIRECTORY_INIT: Once = Once::new();

pub(crate) fn get_program_directory() -> Result<PathBuf, String>
{
  unsafe {
    if let Some(program_dir) = PROGRAM_DIRECTORY.as_ref() {
      return Ok(program_dir.clone());
    }
  }

  fn internal() -> Result<PathBuf, String>
  {
    if let Ok(path_string) = std::env::var(DEFAULT_PROGRAM_DIR_ENV_VARIABLE) {
      match Path::new(&path_string).try_exists() {
        Ok(true) => return Ok(path_string.into()),
        Ok(false) => {
          return Err(format!(
            "Path specified in ${DEFAULT_PROGRAM_DIR_ENV_VARIABLE} \
             (`{path_string}`) does not exist. Please create it manually."
          ));
        }
        Err(err) => {
          return Err(format!("Could not check whether path specified in \
                              ${DEFAULT_PROGRAM_DIR_ENV_VARIABLE} \
                              (`{path_string}`) exists: {err}"));
        }
      }
    }
    let path = get_home_directory()?;
    let dot_program = format!(".{PROGRAM_NAME}");
    debug_println!("{}", path.join(&dot_program).display());
    Ok(path.join(dot_program))
  }

  unsafe {
    let mut err = None;
    PROGRAM_DIRECTORY_INIT.call_once(|| match internal() {
                            Ok(d) => PROGRAM_DIRECTORY = Some(d),
                            Err(e) => err = Some(e),
                          });
    if let Some(msg) = err {
      Err(msg)
    } else {
      Ok(PROGRAM_DIRECTORY.as_ref().expect("directory is set").clone())
    }
  }
}

pub(crate) fn create_program_directory() -> ResultS
{
  let program_path = get_program_directory()?;
  if !program_path.exists() {
    create_dir_all(&program_path).map_err(|err| {
                                   format!("Could not create `{}`: {err}",
                                           program_path.display())
                                 })?;
  }

  if program_path.is_dir() {
    Ok(())
  } else {
    Err("Could not create `{program_path:?}`".to_string())
  }
}

const WEEK: Duration = Duration::from_secs(60 * 60 * 24 * 7);

pub(crate) fn is_docs_json_old() -> Result<bool, String>
{
  let program_path = get_program_directory()?;
  let metadata =
    program_path.join("docs.json").metadata().map_err(|err| err.to_string())?;
  let modified_time = metadata.modified().map_err(|err| err.to_string())?;
  let elapsed_time = SystemTime::now().duration_since(modified_time)
                                      .map_err(|err| err.to_string())?;
  if elapsed_time > WEEK {
    Ok(true)
  } else {
    Ok(false)
  }
}

pub(crate) fn write_to_logfile(message: impl Display)
                               -> Result<PathBuf, String>
{
  let log_file_path = get_program_directory()?.join("logs.txt");
  let mut log_file = if log_file_path.exists() {
                       File::options().append(true).open(&log_file_path)
                     } else {
                       File::create(&log_file_path)
                     }.map_err(|err| {
                        format!("Could not open `{}`: {err}",
                                log_file_path.display())
                      })?;
  writeln!(log_file, "{}", message).map_err(|err| {
                                     format!("Could not write `{}`: {err}",
                                             log_file_path.display())
                                   })?;
  Ok(log_file_path)
}

pub(crate) enum SearchMatch
{
  Exact,
  Vague(Vec<String>),
  None,
}

#[inline]
pub(crate) fn find_docset_in_docs<'a>(docset_name: &str,
                                      docs: &'a [DocsEntry])
                                      -> Option<&'a DocsEntry>
{
  docs.iter().find(|entry| entry.slug == docset_name)
}

// Returns `true` when docset exists in `docs.json`, print a warning otherwise.
pub(crate) fn is_docset_in_docs_or_print_warning(docset_name: &str,
                                                 docs: &[DocsEntry])
                                                 -> bool
{
  match is_docset_in_docs(docset_name, docs) {
    SearchMatch::Exact => return true,
    SearchMatch::Vague(vague_matches) => {
      let end_index = std::cmp::min(3, vague_matches.len());
      let first_three = &vague_matches[..end_index];

      print_warning!("Unknown docset `{docset_name}`. Did you mean `{}`?",
                     first_three.join("`/`"));
    }
    SearchMatch::None => {
      print_warning!("Unknown docset `{docset_name}`. Did you run \
                      `{PROGRAM_NAME} fetch`?");
    }
  }
  false
}

// `exact` is a perfect match, `vague` are files that contain `docset_name` in
// their path.
pub(crate) fn is_docset_in_docs(docset_name: &str,
                                docs: &[DocsEntry])
                                -> SearchMatch
{
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

pub(crate) fn get_docset_mtime(docset_name: &str) -> Result<u64, String>
{
  let mtime_path = get_docset_path(docset_name)?.join(MTIME_FILENAME);
  let mtime_exists =
    mtime_path.try_exists()
              .map_err(|err| {
                format!("Could not check `{}`: {err}", mtime_path.display())
              })?;
  if !mtime_exists {
    // Could not determine mtime. Since that docset belongs to older version of
    // dedoc, assume that the docset is old.
    return Ok(0);
  }

  let mut mtime_file = File::open(&mtime_path).map_err(|err| {
                         format!("Could not open `{}`: {err}",
                                 mtime_path.display())
                       })?;
  let mut mtime_string = String::new();
  let _ = mtime_file.read_to_string(&mut mtime_string).map_err(|err| {
    format!("Could not read `{}`: {err}", mtime_path.display())
  })?;
  let mtime = mtime_string.parse::<u64>().map_err(|err| {
    format!("Could not parse mtime of `{}`: {err}", mtime_path.display())
  })?;

  Ok(mtime)
}

#[inline]
pub(crate) fn is_docset_old(docset_name: &str,
                            docs: &[DocsEntry])
                            -> Result<bool, String>
{
  if let Some(entry) = find_docset_in_docs(docset_name, docs) {
    return Ok(entry.mtime > get_docset_mtime(docset_name)?);
  }
  Err("Docset `{docset_name}` is not downloaded.".to_string())
}

pub(crate) fn get_local_docsets() -> Result<Vec<String>, String>
{
  let docsets_path = get_program_directory()?.join("docsets");
  let docsets_dir_exists =
    docsets_path.try_exists()
                .map_err(|err| {
                  format!("Could not check `{}`: {err}", docsets_path.display())
                })?;

  if !docsets_dir_exists {
    if let Err(err) = create_dir_all(&docsets_path) {
      return Err(format!("Could not create `{}` directory: {err}",
                         docsets_path.display()));
    }
  }
  let docsets_dir = read_dir(docsets_path).map_err(|err| err.to_string())?;

  let mut result = vec![];

  for entry in docsets_dir {
    let entry = entry.map_err(|err| err.to_string())?;

    let holy_result_option_please_stop =
      entry.file_name().to_string_lossy().to_string();

    result.push(holy_result_option_please_stop);
  }

  // Since non-local docsets are sorted alphabetically.
  result.sort();

  Ok(result)
}

#[inline]
pub(crate) fn is_docset_downloaded(docset_name: &String)
                                   -> Result<bool, String>
{
  get_docset_path(docset_name)?
    .try_exists()
    .map_err(|err| format!("Could not check if `{docset_name}` exists: {err}"))
}

#[inline]
pub(crate) fn is_docs_json_exists() -> Result<bool, String>
{
  Ok(get_program_directory()?.join("docs.json").exists())
}

#[inline]
pub(crate) fn get_docset_path(docset_name: &str) -> Result<PathBuf, String>
{
  Ok(get_program_directory()?.join("docsets").join(docset_name))
}
