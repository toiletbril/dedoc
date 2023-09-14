use toiletcli::flags::*;
use toiletcli::flags;

use crate::docs::{deserealize_docs_json, print_page_from_docset, search_docset_in_filenames,
                  search_docset_thoroughly};

use crate::common::ResultS;
use crate::common::{is_docs_json_exists, is_docset_downloaded, is_docset_in_docs, print_search_results};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};

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

    let mut args = args.iter();

    let docset = if let Some(docset_name) = args.next() {
        docset_name
    } else {
        return show_search_help();
    };

    if !is_docset_downloaded(docset)? {
        let message = if is_docset_in_docs(docset, &deserealize_docs_json()?) {
            format!("`{docset}` docset is not downloaded. Try using `download {docset}`.")
        } else {
            format!("`{docset}` does not exist. Try `list` or `fetch`.")
        };
        return Err(message);
    }

    let mut query = args.fold(String::new(), |base, next| base + next + " ");
    query.pop(); // last space

    if flag_precise {
        let (exact_results, vague_results) =
            search_docset_thoroughly(&docset, &query, flag_case_insensitive)?;

        let exact_results_offset = exact_results.len();

        if !flag_open.is_empty() {
            let n = flag_open.parse::<usize>();


            if let Ok(n) = n {
                if n <= exact_results_offset && n > 0 {
                    print_page_from_docset(docset, &exact_results[n - 1])?;
                    return Ok(());
                } else if n - exact_results_offset <= vague_results.len() {
                    print_page_from_docset(docset, &vague_results[n - exact_results_offset - 1])?;
                    return Ok(());
                } else {
                    println!("{YELLOW}WARNING{RESET}: `--open {n}` is out of bounds.");
                }
            } else {
                println!("{YELLOW}WARNING{RESET}: `--open` requires a number.");
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
    } else {
        let results = search_docset_in_filenames(&docset, &query, flag_case_insensitive)?;

        if !flag_open.is_empty() {
            let n = flag_open.parse::<usize>()
                .map_err(|err| format!("Unable to parse --open value as number: {err}"))?;

            if n <= results.len() && n > 0 {
                print_page_from_docset(docset, &results[n - 1])?;
                return Ok(());
            } else {
                println!("{YELLOW}WARNING{RESET}: --open {n} is invalid.");
            }
        }

        if !results.is_empty() {
            println!("{BOLD}Exact matches in `{docset}`{RESET}:");
            print_search_results(results, 1)?;
        } else {
            println!("{BOLD}No exact matches in `{docset}`{RESET}.");
        }
    };

    Ok(())
}
