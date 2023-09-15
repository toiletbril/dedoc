use std::fs::remove_dir_all;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{get_docset_path, is_docset_downloaded, is_name_allowed};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};

fn show_remove_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} remove{RESET} <docset1> [docset2, ..]
    Delete a docset. Only docsets downloaded by {PROGRAM_NAME} can be removed.

{GREEN}OPTIONS{RESET}
        --help                  Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn remove<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;

    let mut flags = flags![
        flag_help: BoolFlag, ["--help"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
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
                    .map_err(|err| format!("Unable to remove {docset_path:?}: {err}"))?;
            }
        } else {
            println!("{YELLOW}WARNING{RESET}: `{docset}` is not installed.");
        }
    }

    Ok(())
}
