use std::process::ExitCode;

extern crate toiletcli;

use toiletcli::common::name_from_path;
use toiletcli::flags::*;
use toiletcli::flags;

mod docs;
use docs::{
    deserealize_docs_json, download_docset_tar_gz, extract_docset_tar_gz, fetch_docs_json,
    print_html_file, print_page_from_docset, search_docset_in_filenames, search_docset_thoroughly,
    serialize_and_overwrite_docs_json,
};

mod common;
use common::{
    is_docs_json_exists, is_docs_json_old, is_docset_downloaded, is_docset_in_docs,
    print_search_results,
};
use common::{BOLD, DEFAULT_DOCS_LINK, GREEN, PROGRAM_NAME, RED, RESET, VERSION, YELLOW};

fn show_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} <subcommand> [args]
    Search DevDocs pages from terminal.

{GREEN}SUBCOMMANDS{RESET}
    {BOLD}fetch{RESET}              Fetch latest available docsets.
    {BOLD}list{RESET}               Show available docsets.
    {BOLD}download{RESET}           Download a docset.
    {BOLD}search{RESET}             List pages that match your query.
    {BOLD}read{RESET}               Display the specified page.

{GREEN}OPTIONS{RESET}
    --help                 Display help message. Can be used with subcommands.
    --version, -v          Display version.

The design is not final, and may be subject to change."
);
    println!("{}", help);
    Ok(())
}

fn show_version() -> Result<(), String> {
    let message = format!(
        "\
dedoc {VERSION}
(c) toiletbril <https://github.com/toiletbril>

Licensed under GPLv3.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law."
    );
    println!("{}", message);
    Ok(())
}

fn show_search_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} search [-ipo] <docset> <query>
    List docset pages that match your query.

{GREEN}OPTIONS{RESET}
    --ignore-case, -i      Ignore character case.
    --precise,     -p      Search more thoroughly and look for mentions in other files.
    --open,        -o <n>  Open n-th exact match.
    --help                 Display help message."
    );
    println!("{}", help);
    Ok(())
}

fn show_read_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} read [-i] <docset> <page>
    Print a page. Pages can be searched using `search`.

{GREEN}OPTIONS{RESET}
    --help               Display help message."
    );
    println!("{}", help);
    Ok(())
}

fn show_fetch_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} fetch [-f]
    Fetch latest `docs.json` which lists available languages and frameworks.

{GREEN}OPTIONS{RESET}
    --force, -f    Update even if `docs.json` is recent.
    --help         Display help message."
    );
    println!("{}", help);
    Ok(())
}

fn show_list_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} list [-a]
    Show available docsets.

{GREEN}OPTIONS{RESET}
    --all,   -a    Show all version-specific docsets.
    --help         Display help message."
    );
    println!("{}", help);
    Ok(())
}

fn show_download_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {PROGRAM_NAME} download [-f] <docset1> [docset2, ..]
    Download a docset. Available docsets can be displayed using `list`.

{GREEN}OPTIONS{RESET}
    --force, -f    Overwrite downloaded docsets.
    --help         Display help message."
    );
    println!("{}", help);
    Ok(())
}

fn entry<Args>(program_name: &String, mut args: &mut Args) -> Result<(), String>
where
    Args: Iterator<Item = String>,
{
    debug!(VERSION);

    let mut flag_version;
    let mut flag_help;

    let mut flags = flags![
        flag_help: BoolFlag,    ["--help"],
        flag_version: BoolFlag, ["--version", "-v"]
    ];

    let subcommand = parse_flags_until_subcommand(&mut args, &mut flags);

    if flag_help {
        return show_help();
    }
    if flag_version {
        return show_version();
    }

    let subcommand = subcommand
        .map_err(|err| format!("{err}. Try `--help` for more information"))?
        .to_lowercase();

    match subcommand.as_str() {
        "f" | "fetch" => {
            let mut flag_help;
            let mut flag_force;

            let mut flags = flags![
                flag_help: BoolFlag,  ["--help"],
                flag_force: BoolFlag, ["--force", "-f"]
            ];

            let args = parse_flags(&mut args, &mut flags)?;
            if flag_help {
                return show_fetch_help();
            }

            if !flag_force && is_docs_json_exists()? && !is_docs_json_old()? {
                let message = format!(
                    "\
{YELLOW}WARNING{RESET}: It seems that your `docs.json` was updated less than a week ago.
{YELLOW}WARNING{RESET}: If you still want to update it, re-run this command with `--force`"
                );
                println!("{}", message);
                return Ok(());
            }

            println!("Fetching `{DEFAULT_DOCS_LINK}`...");
            let docs = fetch_docs_json()?;

            println!("Writing `docs.json`...");
            serialize_and_overwrite_docs_json(docs)?;

            println!("Successfully updated `docs.json`.");
        }
        "l" | "ls" | "list" => {
            let mut flag_help;
            let mut flag_all;

            let mut flags = flags![
                flag_help: BoolFlag, ["--help"],
                flag_all: BoolFlag,  ["--all", "-a"]
            ];

            let args = parse_flags(&mut args, &mut flags)?;
            if flag_help {
                return show_list_help();
            }

            let docs = deserealize_docs_json()?;

            let mut docs_iter = docs.iter().peekable();

            while let Some(entry) = docs_iter.next() {
                // slug has ~ if it's version-specific
                if !flag_all && entry.slug.find("~").is_some() {
                    continue;
                }

                if is_docset_downloaded(&entry.slug)? {
                    print!("{GREEN}{} [downloaded]{RESET}", entry.slug);
                } else {
                    print!("{}", entry.slug);
                }

                if docs_iter.peek().is_some() {
                    print!(", ");
                } else {
                    println!();
                }
            }
        }
        "d" | "dl" | "download" => {
            let mut flag_help;
            let mut flag_force;

            let mut flags = flags![
                flag_help: BoolFlag,  ["--help"],
                flag_force: BoolFlag, ["--force", "-f"]
            ];

            let args = parse_flags(&mut args, &mut flags)?;
            if flag_help {
                return show_download_help();
            }

            if args.is_empty() {
                return Err("No arguments were provided. Try `download --help` for more information".to_string());
            }

            if !is_docs_json_exists()? {
                return Err("`docs.json` does not exist. Please run `fetch` first".to_string());
            }

            let docs = deserealize_docs_json()?;

            let mut args = args.iter();

            while let Some(docset) = args.next() {
                if !flag_force && is_docset_downloaded(docset)? {
                    let message = format!("\
{YELLOW}WARNING{RESET}: `{docset}` is already downloaded. If you still want to update it, re-run this command with `--force`");
                    println!("{}", message);
                    continue;
                } else {
                    if !is_docset_in_docs(docset, &docs) {
                        let message = format!(
                            "\
{YELLOW}WARNING{RESET}: Unknown docset `{docset}`. Did you run `fetch`?"
                        );
                        println!("{}", message);
                        continue;
                    }

                    println!("Downloading `{docset}`...");
                    download_docset_tar_gz(docset, &docs)?;

                    println!("Extracting `{docset}`...");
                    extract_docset_tar_gz(docset)?;

                    println!("Successfully installed `{docset}`.");
                }
            }
        }
        "s" | "ss" | "search" => {
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
            if flag_help {
                return show_search_help();
            }

            let mut args = args.iter();

            let docset = if let Some(_docset) = args.next() {
                _docset
            } else {
                return Err("No docset was provided. Try `search --help` for more information".to_string());
            };

            if !is_docset_downloaded(docset)? {
                let message = format!("`{docset}` docset is not downloaded. Try using `download`");
                return Err(message);
            }

            let mut query = args.fold(String::new(), |base, next| base + next + " ");
            query.pop(); // remove last space

            if flag_precise {
                let (exact, vague) =
                    search_docset_thoroughly(&docset, &query, flag_case_insensitive)?;

                if !flag_open.is_empty() {
                    let n = flag_open.parse::<usize>()
                        .map_err(|err| format!("Unable to parse --open value as number: {err}"))?;

                    if n <= exact.len() && n > 0 {
                        print_html_file(&exact[n - 1])?;
                        return Ok(());
                    } else {
                        println!("{YELLOW}WARNING{RESET}: --open {n} is larger than search result.");
                    }
                }

                if !exact.is_empty() {
                    println!("{BOLD}Exact matches in `{docset}`{RESET}:");
                    print_search_results(exact, &docset)?;
                } else {
                    println!("{BOLD}No exact matches in `{docset}`{RESET}.");
                }

                if !vague.is_empty() {
                    println!("{BOLD}Mentions in other files from `{docset}`{RESET}:");
                    print_search_results(vague, &docset)?;
                } else {
                    println!("{BOLD}No mentions in other files from `{docset}`{RESET}.");
                }
            } else {
                let result = search_docset_in_filenames(&docset, &query, flag_case_insensitive)?;

                if !flag_open.is_empty() {
                    let n = flag_open.parse::<usize>()
                        .map_err(|err| format!("Unable to parse --open value as number: {err}"))?;

                    if n <= result.len() && n > 0 {
                        print_html_file(&result[n - 1])?;
                        return Ok(());
                    } else {
                        println!("{YELLOW}WARNING{RESET}: --open {n} is invalid.");
                    }
                }

                if !result.is_empty() {
                    println!("{BOLD}Exact matches in `{docset}`{RESET}:");
                    print_search_results(result, &docset)?;
                } else {
                    println!("{BOLD}No exact matches in `{docset}`{RESET}.");
                }
            };
        }
        "r" | "read" => {
            let mut flag_help;

            let mut flags = flags![
                flag_help: BoolFlag, ["--help"]
            ];

            let args = parse_flags(&mut args, &mut flags)?;
            if flag_help {
                return show_read_help();
            }

            let mut args = args.iter();

            let docset = if let Some(_docset) = args.next() {
                _docset
            } else {
                return Err("No docset was provided. Try `read --help` for more information.".to_string());
            };

            if !is_docset_downloaded(docset)? {
                let message = format!("`{docset}` docset is not downloaded. Try using `download`");
                return Err(message);
            }

            let mut query = args.fold(String::new(), |base, next| base + next + " ");
            query.pop(); // remove last space

            if query.is_empty() {
                return Err("No page specified. Try `read --help` for more information.".to_string());
            }

            print_page_from_docset(docset, &query)?;
        }
        other => return Err(format!("Unknown subcommand `{other}`")),
    }

    Ok(())
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let program_name = name_from_path(&args.next().expect("Progran path is provided"));

    match entry(&program_name, &mut args) {
        Err(err) => {
            eprintln!("{RED}ERROR{RESET}: {err}");
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}
