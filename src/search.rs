use std::fs::File;
use std::io::{BufWriter, BufReader, Read};

use serde::{Serialize, Deserialize};
use serde_json::{from_str, to_writer};

use toiletcli::flags::*;
use toiletcli::flags;

use crate::{docs::{deserialize_docs_json, print_page_from_docset, search_docset_in_filenames,
                  search_docset_thoroughly}, debug_println};

use crate::common::ResultS;
use crate::common::{is_docs_json_exists, is_docset_downloaded, is_docset_in_docs, print_search_results,
                   get_program_directory};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};

#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
struct SearchFlags {
    case_insensitive: bool,
    precise: bool
}

#[derive(Serialize, Deserialize)]
struct SearchCache {
    query: String,
    docset: String,
    exact_items: Vec<String>,
    vague_items: Vec<String>,
    flags: SearchFlags
}

fn try_use_cache(docset: &String, query: &String, flags: &SearchFlags) -> Option<SearchCache> {
    let program_dir = get_program_directory().ok()?;
    let cache_path = program_dir.join("search_cache.json");

    let file = File::open(cache_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut string_buffer = vec![];

    reader.read_to_end(&mut string_buffer).ok()?;
    let contents = String::from_utf8(string_buffer).ok()?;

    let cache: SearchCache = from_str(&contents).ok()?;

    if docset == &cache.docset && query == &cache.query && flags == &cache.flags {
        Some(cache)
    } else {
        None
    }
}

fn write_search_cache(
    docset: &String,
    query: &String,
    flags: &SearchFlags,
    exact_items: &Vec<String>,
    vague_items: &Vec<String>) -> ResultS
{
    let program_dir = get_program_directory()?;
    let cache_path = program_dir.join("search_cache.json");

    let cache_file = File::create(&cache_path)
        .map_err(|err| format!("Could not open {cache_path:?}: {err}"))?;

    let writer = BufWriter::new(cache_file);

    let cache = SearchCache {
        docset:      docset.clone(),
        query:       query.clone(),
        flags:       flags.clone(),
        exact_items: exact_items.clone(),
        vague_items: vague_items.clone()
    };

    to_writer(writer, &cache)
        .map_err(|err| format!("Could not write cache: {err}"))?;

    Ok(())
}

fn show_search_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} search{RESET} [-ipo] <docset> <query>
    List docset pages that match your query.

{GREEN}OPTIONS{RESET}
    -i, --ignore-case           Ignore character case.
    -p, --precise               Search more thoroughly and look for mentions in other files.
    -o, --open <number>         Open n-th search result.
        --help                  Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn search<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_precise;
    let mut flag_open;
    let mut flag_case_insensitive;

    let mut flags = flags![
        flag_help: BoolFlag,             ["--help"],
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

    if !is_docset_downloaded(&docset)? {
        let message = if is_docset_in_docs(&docset, &deserialize_docs_json()?) {
            format!("Docset `{docset}` is not downloaded. Try using `download {docset}`.")
        } else {
            format!("Docset `{docset}` does not exist. Try `list` or `fetch`.")
        };
        return Err(message);
    }

    let query = args.collect::<Vec<String>>().join(" ");
    let flag_open_is_empty = flag_open.is_empty();
    let open_number = flag_open.parse::<usize>().ok();

    let flags = SearchFlags { precise: flag_precise, case_insensitive: flag_case_insensitive };

    if flag_open_is_empty {
        // Printing query is needed to let you know if you messed up any flags
        println!("Searching for `{query}`...");
    }

    if flag_precise {
        let (exact_results, vague_results) =
        if let Some(cache) = try_use_cache(&docset, &query, &flags) {
            debug_println!("Search used cache.");
            (cache.exact_items, cache.vague_items)
        } else {
            let (exact, vague) = search_docset_thoroughly(&docset, &query, flag_case_insensitive)?;
            if let Err(err) = write_search_cache(&docset, &query, &flags, &exact, &vague) {
                println!("{YELLOW}WARNING{RESET}: Could not write cache: {err}.");
            }
            (exact, vague)
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
            print_search_results(exact_results, 1)?;
        } else {
            println!("{BOLD}No exact matches in `{docset}`{RESET}.");
        }

        if !vague_results.is_empty() {
            println!("{BOLD}Mentions in other files from `{docset}`{RESET}:");
            print_search_results(vague_results, exact_results_offset + 1)?;
        } else {
            println!("{BOLD}No mentions in other files from `{docset}`{RESET}.");
        }

        return Ok(());
    } else {
        let results =
        if let Some(cache) = try_use_cache(&docset, &query, &flags) {
            debug_println!("Using cache");
            cache.exact_items
        } else {
            let exact = search_docset_in_filenames(&docset, &query, flag_case_insensitive)?;
            if let Err(err) = write_search_cache(&docset, &query, &flags, &exact, &vec![]) {
                println!("{YELLOW}WARNING{RESET}: Could not write cache: {err}.");
            }
            exact
        };

        if !flag_open_is_empty {
            if let Some(n) = open_number {
                if n <= results.len() && n > 0 {
                    return print_page_from_docset(&docset, &results[n - 1]);
                } else {
                    println!("{YELLOW}WARNING{RESET}: --open {n} is out of bounds.");
                }
            } else {
                println!("{YELLOW}WARNING{RESET}: `--open` requires a number.");
            }
        }

        if !results.is_empty() {
            println!("{BOLD}Exact matches in `{docset}`{RESET}:");
            return print_search_results(results, 1);
        } else {
            println!("{BOLD}No exact matches in `{docset}`{RESET}.");
        }
    }

    Ok(())
}
