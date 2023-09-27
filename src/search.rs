use std::borrow::Cow;
use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
    deserialize_docs_json, get_docset_path, get_program_directory, is_docs_json_exists,
    is_docset_in_docs_or_print_warning, print_page_from_docset, is_docset_downloaded
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, GRAY, RESET, YELLOW, DOC_PAGE_EXTENSION};

fn show_search_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} search{RESET} [-wipo] <docset> <query>
    List docset pages that match your query.

{GREEN}OPTIONS{RESET}
    -w, --whole                     Search for the whole sentence.
    -i, --ignore-case               Ignore character case.
    -p, --precise                   Look inside files (like 'grep').
    -o, --open <number>             Open n-th result.
        --help                      Display help message."
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ExactResult {
    item: String,
    fragment: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct VagueResult {
    item: String,
    contexts: Vec<String>,
}

// Flags that change search result must be added here for cache to be updated.
#[derive(Serialize, Deserialize)]
#[derive(Default, PartialEq, Clone)]
struct SearchFlags {
    case_insensitive: bool,
    precise: bool,
    whole: bool,
}

// Sometimes search results are big, and it's cheaper to check a small file if current search
// options match cached ones, to deserialize the whole search cache.
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
struct SearchOptions<'a> {
    query:  Cow<'a, str>,
    docset: Cow<'a, str>,
    flags:  Cow<'a, SearchFlags>,
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
struct SearchCache<'a> {
    exact_items:   Cow<'a, [ExactResult]>,
    vague_matches: Cow<'a, [VagueResult]>,
}

fn try_use_cache<'a>(search_options: &SearchOptions) -> Option<SearchCache<'a>> {
    let program_dir = get_program_directory().ok()?;
    let cache_header_path = program_dir.join("search_cache_header.json");

    {
        let cache_options_file = File::open(cache_header_path).ok()?;
        let cache_options_reader = BufReader::new(cache_options_file);

        let cached_search_options: SearchOptions = from_reader(cache_options_reader).ok()?;

        if cached_search_options != *search_options {
            return None;
        }
    }

    let cache_path = program_dir.join("search_cache.json");

    let cache_file = File::open(cache_path).ok()?;
    let cache_reader = BufReader::new(cache_file);

    let cache: SearchCache = from_reader(cache_reader).ok()?;

    Some(cache)
}

fn cache_search_results(
    search_options: &SearchOptions,
    search_cache:   &SearchCache,
) -> ResultS {
    let program_dir = get_program_directory()?;

    {
        let cache_options_path = program_dir.join("search_cache_header.json");
        let cache_options_file = File::create(&cache_options_path)
            .map_err(|err| format!("Could not open cache options at `{}`: {err}", cache_options_path.display()))?;

        let cache_options_writer = BufWriter::new(cache_options_file);

        to_writer(cache_options_writer, &search_options)
            .map_err(|err| format!("Could not write cache options at `{}`: {err}", cache_options_path.display()))?;
    }

    {
        let cache_path = program_dir.join("search_cache.json");
        let cache_file = File::create(&cache_path)
            .map_err(|err| format!("Could not open cache at `{}`: {err}", cache_path.display()))?;

        let cache_writer = BufWriter::new(cache_file);

        to_writer(cache_writer, &search_cache)
            .map_err(|err| format!("Could not write cache at `{}`: {err}", cache_path.display()))?;
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
struct IndexEntry {
    name: String,
    path: String,
    #[serde(skip)]
    r#type: String,
}

#[derive(Deserialize)]
struct IndexJson {
    entries: Vec<IndexEntry>
}

type ExactMatches = Vec<ExactResult>;
type VagueMatches = Vec<VagueResult>;

pub fn search_docset_in_filenames(
    docset_name: &String,
    query: &String,
    case_insensitive: bool,
) -> Result<ExactMatches, String> {
    let docset_path = get_docset_path(docset_name)?;
    let index_json_path = docset_path.join("index.json");

    let index_exists = index_json_path.try_exists()
        .map_err(|err| format!("Could not check `{}`: {err}", index_json_path.display()))?;

    if !index_exists {
        let message = format!("\
Index file does not exist for `{docset_name}`. Docsets that were downloaded prior to version `0.2.0` are incompatible. \
Please redownload the docset with `download {docset_name} --force`."
        );
        return Err(message);
    }

    let file = File::open(&index_json_path)
        .map_err(|err| format!("Could not open `{}`: {err}", index_json_path.display()))?;

    let reader = BufReader::new(file);

    let index: IndexJson = from_reader(reader)
        .map_err(|err| format!("Could not deserialize `{}`: {err}", index_json_path.display()))?;

    let mut items = vec![];

    if case_insensitive {
        let query = query.to_lowercase();

        for entry in index.entries {
            let name = entry.name.to_lowercase();
            let path = entry.path.to_lowercase();

            if name.contains(&query) || path.contains(&query) {
                let mut path_split = path.split('#');

                let item = if let Some(item) = path_split.next() {
                    Ok(item)
                } else {
                    Err(format!("Invalid page path: {}", entry.path))
                }?
                .to_owned();

                let fragment = path_split.next()
                    .map(|s| s.to_owned());

                let exact_match = ExactResult { item, fragment };

                items.push(exact_match);
            }
        }
    } else {
        for entry in index.entries {
            if entry.name.contains(query) || entry.path.contains(query) {
                let mut path_split = entry.path.split('#');

                let item = if let Some(item) = path_split.next() {
                    Ok(item)
                } else {
                    Err(format!("Invalid page path: {}", entry.path))
                }?
                .to_owned();

                let fragment = path_split.next()
                    .map(|s| s.to_owned());

                let exact_match = ExactResult { item, fragment };

                items.push(exact_match);
            }
        }
    }

    items.sort_unstable();

    Ok(items)
}

fn get_context_around_query(html_line: &String, index: usize, query_len: usize) -> String {
    const BOUND_OFFSET: usize = 37;

    let lower_bound = index.saturating_sub(BOUND_OFFSET);
    let upper_bound = (index + query_len).saturating_add(BOUND_OFFSET);
    let word_end_index = index + query_len;

    let start_pos = html_line.char_indices()
        .rev()
        .find(|&(idx, _)| idx <= lower_bound)
        .map_or(0, |(idx, _)| idx);

    let end_pos = html_line.char_indices()
        .skip_while(|&(idx, _)| idx < word_end_index)
        .find(|&(idx, _)| idx >= upper_bound)
        .map_or(html_line.len(), |(idx, _)| idx);

    html_line[start_pos..end_pos].trim().to_owned()
}

// Item is a file path without a file extension which is relative to docset directory
pub fn convert_path_to_item(path: PathBuf, docset_path: &PathBuf) -> Result<String, String> {
    let item = path
        .strip_prefix(&docset_path)
        .map_err(|err| err.to_string())?;
    let item = item.with_extension("");

    Ok(item.display().to_string())
}

pub fn search_docset_precisely(
    docset_name: &String,
    query: &String,
    case_insensitive: bool,
) -> Result<(ExactMatches, VagueMatches), String> {
    let docset_path = get_docset_path(docset_name)?;

    let internal_query = if case_insensitive {
        query.to_lowercase()
    } else {
        query.to_owned()
    };

    fn visit_dir_with_query(
        original_path: &PathBuf,
        path: &PathBuf,
        query: &String,
        case_insensitive: bool,
    ) -> Result<(ExactMatches, VagueMatches), String> {
        let mut exact_files   = vec![];
        let mut vague_results = vec![];

        let dir = read_dir(&path)
            .map_err(|err| format!("Could not read `{}` directory: {err}", path.display()))?;

        for entry in dir {
            let entry = entry
                .map_err(|err| format!("Could not read file: {err}"))?;

            let os_file_name = entry.file_name();

            let file_type = entry
                .file_type()
                .map_err(|err| format!("Could not read file type of {os_file_name:?}: {err}"))?;

            if file_type.is_dir() {
                let (mut exact, mut vague) =
                    visit_dir_with_query(original_path, &entry.path(), &query, case_insensitive)?;

                exact_files.append(&mut exact);
                vague_results.append(&mut vague);
            }

            let mut file_name = os_file_name.to_string_lossy().to_string();

            if !file_name.ends_with(DOC_PAGE_EXTENSION) {
                continue;
            }

            if case_insensitive {
                file_name.make_ascii_lowercase();
            }

            let file_path = entry.path();

            if file_name.contains(query) {
                let item = convert_path_to_item(file_path, original_path)?;
                let exact_match = ExactResult { item, fragment: None };
                exact_files.push(exact_match);
            } else {
                let file = File::open(&file_path)
                    .map_err(|err| format!("Could not open `{}`: {err}", file_path.display()))?;

                let query_len = query.len();

                let mut contexts = vec![];

                let mut reader = BufReader::new(file);
                let mut string_buffer = String::new();

                while let Ok(size) = reader.read_line(&mut string_buffer) {
                    if size == 0 {
                        break;
                    }

                    if let Some(index) = string_buffer.find(query) {
                        let context = get_context_around_query(&string_buffer, index, query_len);
                        contexts.push(context);
                    }

                    string_buffer.clear();
                }

                if !contexts.is_empty() {
                    let vague_result = VagueResult {
                        item: convert_path_to_item(file_path, original_path)?,
                        contexts: contexts,
                    };

                    vague_results.push(vague_result);
                }
            }
        }
        Ok((exact_files, vague_results))
    }

    let (mut exact_files, mut vague_results) =
        visit_dir_with_query(&docset_path, &docset_path, &internal_query, case_insensitive)?;

    exact_files.sort_unstable();
    vague_results.sort_unstable();

    let items = (
        exact_files,
        vague_results,
    );

    Ok(items)
}

pub fn print_vague_search_results(search_results: &[VagueResult], mut start_index: usize) -> ResultS {
    for result in search_results {
        println!("{GRAY}{start_index:>4}{RESET}  {}{GRAY}", result.item);

        for context in &result.contexts {
            println!("          {GRAY}...{}...{RESET}", context);
        }

        start_index += 1;
    }

    Ok(())
}

pub fn print_search_results(search_results: &[ExactResult], mut start_index: usize) -> ResultS {
    let mut prev_item = "";

    // Group fragments by an item.
    for result in search_results {
        if let Some(fragment) = &result.fragment {
            if result.item == prev_item {
                println!("          {GRAY}#{}{RESET}", fragment);
            } else {
                println!("{GRAY}{start_index:>4}{RESET}  {}{GRAY}, #{}{RESET}", result.item, fragment);
            }
        } else {
            println!("{GRAY}{start_index:>4}{RESET}  {}", result.item);
        }

        prev_item = &result.item;
        start_index += 1;
    }

    Ok(())
}

pub fn search<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_whole;
    let mut flag_precise;
    let mut flag_open;
    let mut flag_case_insensitive;

    let mut flags = flags![
        flag_help: BoolFlag,             ["--help"],
        flag_whole: BoolFlag,            ["--whole", "-w"],
        flag_precise: BoolFlag,          ["--precise", "-p"],
        flag_open: StringFlag,           ["--open", "-o"],
        flag_case_insensitive: BoolFlag, ["--ignore-case", "-i"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_search_help(); }

    if !is_docs_json_exists()? {
        return Err("`docs.json` does not exist. Maybe run `fetch` first?".to_string());
    }

    let mut args = args.into_iter();

    let docset = if let Some(docset_name) = args.next() {
        docset_name
    } else {
        return show_search_help();
    };

    let docs = deserialize_docs_json()?;

    if !is_docset_downloaded(&docset)? {
        if is_docset_in_docs_or_print_warning(&docset, &docs) {
            println!("\
{YELLOW}WARNING{RESET}: Docset `{docset}` is not downloaded. Try running `download {docset}`."
            );
        }
        return Ok(());
    }

    let query = {
        let mut query = args.collect::<Vec<String>>().join(" ");
        if flag_whole {
            query.insert(0, ' ');
            query.push(' ');
            query
        } else {
            query
        }
    };

    let flag_open_is_empty = flag_open.is_empty();
    let open_number = flag_open.parse::<usize>().ok();

    let search_flags = SearchFlags {
        precise: flag_precise,
        case_insensitive: flag_case_insensitive,
        whole: flag_whole,
    };

    let search_options = SearchOptions {
        query:  Cow::Borrowed(&query),
        docset: Cow::Borrowed(&docset),
        flags:  Cow::Borrowed(&search_flags),
    };

    if flag_open_is_empty {
        // Printing query is needed to let you know if you messed up any flags
        println!("Searching for `{query}`...");
    }

    if flag_precise {
        let (exact_results, vague_results) =
        if let Some(cache) = try_use_cache(&search_options) {
            (cache.exact_items, cache.vague_matches)
        } else {
            let (exact, vague) = search_docset_precisely(&docset, &query, flag_case_insensitive)?;

            let search_cache = SearchCache {
                exact_items:   Cow::Borrowed(&exact),
                vague_matches: Cow::Borrowed(&vague),
            };

            let _ = cache_search_results(&search_options, &search_cache)
                .map_err(|err| format!("{YELLOW}WARNING{RESET}: Could not write cache: {err}."));

            (exact.into(), vague.into())
        };

        let exact_results_offset = exact_results.len();

        if !flag_open_is_empty {
            match open_number {
                Some(n) if n < 1 || n > exact_results_offset + vague_results.len() => {
                    println!("{YELLOW}WARNING{RESET}: `--open {n}` is out of bounds.");
                }
                Some(n) if n <= exact_results_offset => {
                    return print_page_from_docset(&docset, &exact_results[n - 1].item);
                }
                Some(n) => {
                    return print_page_from_docset(&docset, &vague_results[n - exact_results_offset - 1].item);
                }
                _ => {
                    println!("{YELLOW}WARNING{RESET}: `--open` requires a number.");
                }
            }
        }

        if !exact_results.is_empty() {
            println!("{BOLD}Exact matches in `{docset}`{RESET}:");
            print_search_results(&exact_results, 1)?;
        } else {
            println!("{BOLD}No exact matches in `{docset}`{RESET}.");
        }

        if !vague_results.is_empty() {
            println!("{BOLD}Mentions in other files from `{docset}`{RESET}:");
            print_vague_search_results(&vague_results, exact_results_offset + 1)?;
        } else {
            println!("{BOLD}No mentions in other files from `{docset}`{RESET}.");
        }

        return Ok(());
    } else {
        let results = if let Some(cache) = try_use_cache(&search_options) {
            cache.exact_items
        } else {
            let exact  = search_docset_in_filenames(&docset, &query, flag_case_insensitive)?;

            let search_cache = SearchCache {
                exact_items:   Cow::Borrowed(&exact),
                vague_matches: Cow::Owned(vec![]),
            };

            let _ = cache_search_results(&search_options, &search_cache)
                .map_err(|err| format!("{YELLOW}WARNING{RESET}: Could not write cache: {err}."));

            exact.into()
        };

        if !flag_open_is_empty {
            match open_number {
                Some(n) if n < 1 || n > results.len() => {
                    println!("{YELLOW}WARNING{RESET}: `--open {n}` is out of bounds.");
                }
                Some(n) => {
                    return print_page_from_docset(&docset, &results[n - 1].item);
                }
                _ => {
                    println!("{YELLOW}WARNING{RESET}: `--open` requires a number.");
                }
            }
        }

        if !results.is_empty() {
            println!("{BOLD}Exact matches in `{docset}`{RESET}:");
            return print_search_results(&results, 1);
        } else {
            println!("{BOLD}No exact matches in `{docset}`{RESET}.");
        }
    }

    Ok(())
}
