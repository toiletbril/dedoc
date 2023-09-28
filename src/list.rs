use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{deserialize_docs_json, get_local_docsets, is_docs_json_exists};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_list_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} list{RESET} [-lan]
    Show available docsets.

{GREEN}OPTIONS{RESET}
    -l, --local                     Show only local docsets.
    -a, --all                       Show all version-specific docsets.
    -n, --newlines                  Print each docset on a separate line.
        --help                      Display help message."
    );
    Ok(())
}

pub fn list<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_all;
    let mut flag_local;
    let mut flag_newlines;

    let mut flags = flags![
        flag_help: BoolFlag,     ["--help"],
        flag_all: BoolFlag,      ["--all", "-a"],
        flag_local: BoolFlag,    ["--local", "-l"],
        flag_newlines: BoolFlag, ["--newlines", "-n"]
    ];

    parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_list_help(); }

    if !is_docs_json_exists()? {
        return Err("The list of available documents has not yet been downloaded. Please run `fetch` first.".to_string());
    }

    let local_docsets = get_local_docsets()?;

    let separator = if flag_newlines { "\n" } else { ", " };

    if flag_local {
        let mut local_docsets_iter_peekable = local_docsets.iter().peekable();

        while let Some(entry) = local_docsets_iter_peekable.next() {
            print!("{GREEN}{} [downloaded]{RESET}", entry);

            if local_docsets_iter_peekable.peek().is_some() {
                print!("{}", separator);
            } else {
                println!();
            }
        }

        return Ok(());
    }

    let docs = deserialize_docs_json()?;
    let docs_names = docs
        .iter()
        .map(|entry| entry.slug.to_string())
        .collect::<Vec<String>>();

    let mut docs_names_peekable = docs_names.iter().peekable();

    while let Some(entry) = docs_names_peekable.next() {
        // slug has ~ if it's version-specific
        if !flag_local && !flag_all && entry.find("~").is_some() {
            continue;
        }

        if local_docsets.contains(entry) {
            print!("{GREEN}{} [downloaded]{RESET}", entry);
        } else {
            print!("{}", entry);
        }

        if docs_names_peekable.peek().is_some() {
            print!("{}", separator);
        } else {
            println!();
        }
    }

    Ok(())
}
