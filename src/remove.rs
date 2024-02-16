use std::fs::remove_dir_all;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{get_docset_path, is_docset_downloaded, get_local_docsets, get_flag_error};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};
use crate::print_warning;

fn show_remove_help() -> ResultS {
    println!("\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} remove{RESET} <docset1> [docset2, ...]
    Delete a docset. Only docsets downloaded by {PROGRAM_NAME} can be removed.

{GREEN}OPTIONS{RESET}
        --purge-all                 Remove all installed docsets.
        --help                      Display help message.");
    Ok(())
}

fn is_name_allowed(docset_name: &str) -> bool {
    let has_slashes = {
        #[cfg(target_family = "windows")]
        { docset_name.contains('\\') || docset_name.contains('/') }

        #[cfg(target_family = "unix")]
        { docset_name.contains('/') }
    };

    let is_bad = has_slashes ||
        docset_name.starts_with('~') ||
        docset_name.contains('$') ||
        docset_name.starts_with('.') ||
        docset_name.contains("..");

    !is_bad
}

pub(crate) fn remove<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_purge_all;
    let mut flag_help;

    let mut flags = flags![
        flag_help: BoolFlag,      ["--help"],
        flag_purge_all: BoolFlag, ["--purge-all"]
    ];

    let args = parse_flags(&mut args, &mut flags)
        .map_err(|err| get_flag_error(&err))?;

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
            print_warning!("`{docset}` contains forbidden characters.");
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
            print_warning!("{YELLOW}WARNING{RESET}: `{docset}` is not installed.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_names() {
        let bad_name_path = "/what";
        let bad_name_home = "~";
        let bad_name_dots = "..";
        let bad_name_env  = "$HOME";

        let good_name_simple  = "hello";
        let good_name_version = "qt~6.1";
        let good_name_long    = "scala~2.13_reflection";

        assert!(!is_name_allowed(&bad_name_path));
        assert!(!is_name_allowed(&bad_name_home));
        assert!(!is_name_allowed(&bad_name_dots));
        assert!(!is_name_allowed(&bad_name_env));

        assert!(is_name_allowed(&good_name_simple));
        assert!(is_name_allowed(&good_name_version));
        assert!(is_name_allowed(&good_name_long));
    }
}
