use std::fs::remove_dir_all;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{get_docset_path, is_docset_downloaded, is_name_allowed, get_local_docsets};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};

fn show_remove_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} remove{RESET} <docset1> [docset2, ...]
    Delete a docset. Only docsets downloaded by {PROGRAM_NAME} can be removed.

{GREEN}OPTIONS{RESET}
        --purge-all                 Remove all installed docsets.
        --help                      Display help message."
    );
    Ok(())
}

pub fn remove<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_purge_all;

    let mut flags = flags![
        flag_help: BoolFlag,      ["--help"],
        flag_purge_all: BoolFlag, ["--purge-all"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;

    if flag_purge_all {
        let local_docsets = get_local_docsets()?;
        for docset in local_docsets {
            let docset_path = get_docset_path(&docset)?;
            println!("Removing `{docset}` from `{}`...", docset_path.display());
            remove_dir_all(&docset_path)
                .map_err(|err| format!("Unable to remove `{}`: {err}", docset_path.display()))?;
        }
        return Ok(());
    }

    if flag_help || args.is_empty() { return show_remove_help(); }

    for docset in args.iter() {
        if !is_name_allowed(docset) {
            println!("{YELLOW}WARNING{RESET}: `{docset}` contains forbidden characters.");
            continue;
        }

        if is_docset_downloaded(docset)? {
            let docset_path = get_docset_path(docset)?;
            if docset_path.exists() {
                println!("Removing `{docset}` from `{}`...", docset_path.display());
                remove_dir_all(&docset_path)
                    .map_err(|err| format!("Unable to remove `{}`: {err}", docset_path.display()))?;
            }
        } else {
            println!("{YELLOW}WARNING{RESET}: `{docset}` is not installed.");
        }
    }

    Ok(())
}
