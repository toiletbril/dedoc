use toiletcli::flags::*;
use toiletcli::flags;

use crate::docs::deserialize_docs_json;

use crate::common::ResultS;
use crate::common::{is_docs_json_exists, get_local_docsets};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_list_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} list{RESET} [-la]
    Show available docsets.

{GREEN}OPTIONS{RESET}
    -l, --local                     Only show local docsets.
    -a, --all                       Show all version-specific docsets.
        --help                      Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn list<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_all;
    let mut flag_local;

    let mut flags = flags![
        flag_help: BoolFlag,  ["--help"],
        flag_all: BoolFlag,   ["--all", "-a"],
        flag_local: BoolFlag, ["--local", "-l"]
    ];

    parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_list_help(); }

    if !is_docs_json_exists()? {
        return Err("`docs.json` does not exist. Maybe run `fetch` first?".to_string());
    }

    if flag_local {
        let local_docsets = get_local_docsets()?;
        let mut local_docsets_iter_peekable = local_docsets
            .iter()
            .peekable();

        while let Some(entry) = local_docsets_iter_peekable.next() {
            print!("{GREEN}{} [downloaded]{RESET}", entry);
            if local_docsets_iter_peekable.peek().is_some() {
                print!(", ");
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

        let local_docsets = get_local_docsets()?;

        if local_docsets.contains(entry) {
            print!("{GREEN}{} [downloaded]{RESET}", entry);
        } else {
            print!("{}", entry);
        }

        if docs_names_peekable.peek().is_some() {
            print!(", ");
        } else {
            println!();
        }
    }

    Ok(())
}