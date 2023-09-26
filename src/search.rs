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
    convert_paths_to_items, deserialize_docs_json, get_docset_path, get_program_directory,
    is_docs_json_exists, is_docset_in_docs_or_print_warning, print_page_from_docset,
    print_search_results, is_docset_downloaded
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW, DOC_PAGE_EXTENSION};

fn show_search_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} search{RESET} [-ipo] <docset> <query>
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

// Flags that change search result must be added here for cache to be updated.
#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
struct SearchFlags {
    case_insensitive: bool,
    precise: bool,
    whole: bool
}

#[derive(Serialize, Deserialize, PartialEq)]
struct SearchCache<'a> {
    query:       Cow<'a, str>,
    docset:      Cow<'a, str>,
    exact_items: Cow<'a, [String]>,
    vague_items: Cow<'a, [String]>,
    flags:       Cow<'a, SearchFlags>,
}

fn try_use_cache<'a>(docset: &'a String, query: &'a String, flags: &'a SearchFlags) -> Option<SearchCache<'a>> {
    let program_dir = get_program_directory().ok()?;
    let cache_path = program_dir.join("search_cache.json");

    let file = File::open(cache_path).ok()?;
    let reader = BufReader::new(file);

    let cache: SearchCache = from_reader(reader).ok()?;

    if docset == &cache.docset && query == &cache.query && *flags == *cache.flags {
        Some(cache)
    } else {
        None
    }
}

fn cache_search_results(
    docset: &String,
    query:  &String,
    flags:  &SearchFlags,
    exact_items: &Vec<String>,
    vague_items: &Vec<String>,
) -> ResultS {
    let program_dir = get_program_directory()?;
    let cache_path = program_dir.join("search_cache.json");

    let cache_file = File::create(&cache_path)
        .map_err(|err| format!("Could not open cache at `{}`: {err}", cache_path.display()))?;

    let writer = BufWriter::new(cache_file);

    let cache = SearchCache {
        docset:      Cow::Borrowed(docset),
        query:       Cow::Borrowed(query),
        flags:       Cow::Borrowed(flags),
        exact_items: Cow::Borrowed(exact_items),
        vague_items: Cow::Borrowed(vague_items),
    };

    to_writer(writer, &cache)
        .map_err(|err| format!("Could not write cache at `{}`: {err}", cache_path.display()))?;

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

pub fn search_docset_in_filenames(
    docset_name: &String,
    query: &String,
    case_insensitive: bool,
) -> Result<Vec<String>, String> {
    let docset_path = get_docset_path(docset_name)?;
    let index_json_path = docset_path.join("index.json");

    let index_exists = index_json_path.try_exists()
        .map_err(|err| format!("Could not check `{}`: {err}", index_json_path.display()))?;

    if !index_exists {
        let message = format!("\
Index file does not exist. Dedoc docsets that were downloaded prior to `0.2.0` version did not use them. \
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
                items.push(entry.path);
            }
        }
    } else {
        for entry in index.entries {
            if entry.name.contains(query) || entry.path.contains(query) {
                items.push(entry.path);
            }
        }
    }

    items.sort_unstable();

    Ok(items)
}

type ExactMatches = Vec<String>;
type VagueMatches = Vec<String>;

pub fn search_docset_thoroughly(
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
        path: &PathBuf,
        query: &String,
        case_insensitive: bool,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let mut exact_paths = vec![];
        let mut vague_paths = vec![];

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
                    visit_dir_with_query(&entry.path(), &query, case_insensitive)?;
                exact_paths.append(&mut exact);
                vague_paths.append(&mut vague);
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
                exact_paths.push(file_path);
            } else {
                let file = File::open(&file_path)
                    .map_err(|err| format!("Could not open `{}`: {err}", file_path.display()))?;
                let mut reader = BufReader::new(file);
                let mut string_buffer = String::new();

                while let Ok(size) = reader.read_line(&mut string_buffer) {
                    if size == 0 {
                        break;
                    }

                    if string_buffer.contains(query) {
                        vague_paths.push(entry.path());
                        break;
                    }

                    string_buffer.clear();
                }
            }
        }
        Ok((exact_paths, vague_paths))
    }

    let (exact_paths, vague_paths) =
        visit_dir_with_query(&docset_path, &internal_query, case_insensitive)?;

    let mut items = (
        convert_paths_to_items(exact_paths, docset_name)?,
        convert_paths_to_items(vague_paths, docset_name)?,
    );

    items.0.sort_unstable();
    items.1.sort_unstable();

    Ok(items)
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

    let flags = SearchFlags {
        precise: flag_precise,
        case_insensitive: flag_case_insensitive,
        whole: flag_whole,
    };

    if flag_open_is_empty {
        // Printing query is needed to let you know if you messed up any flags
        println!("Searching for `{query}`...");
    }

    if flag_precise {
        let (exact_results, vague_results) =
        if let Some(cache) = try_use_cache(&docset, &query, &flags) {
            (cache.exact_items, cache.vague_items)
        } else {
            let (exact, vague) = search_docset_thoroughly(&docset, &query, flag_case_insensitive)?;
            let _ = cache_search_results(&docset, &query, &flags, &exact, &vague)
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
                    return print_page_from_docset(&docset, &exact_results[n - 1]);
                }
                Some(n) => {
                    return print_page_from_docset(&docset, &vague_results[n - exact_results_offset - 1]);
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
            print_search_results(&vague_results, exact_results_offset + 1)?;
        } else {
            println!("{BOLD}No mentions in other files from `{docset}`{RESET}.");
        }

        return Ok(());
    } else {
        let results = if let Some(cache) = try_use_cache(&docset, &query, &flags) {
            cache.exact_items
        } else {
            let exact = search_docset_in_filenames(&docset, &query, flag_case_insensitive)?;
            let _ = cache_search_results(&docset, &query, &flags, &exact, &vec![])
                .map_err(|err| format!("{YELLOW}WARNING{RESET}: Could not write cache: {err}."));
            exact.into()
        };

        if !flag_open_is_empty {
            match open_number {
                Some(n) if n < 1 || n > results.len() => {
                    println!("{YELLOW}WARNING{RESET}: `--open {n}` is out of bounds.");
                }
                Some(n) => {
                    return print_page_from_docset(&docset, &results[n - 1]);
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
