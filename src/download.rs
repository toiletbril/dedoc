use toiletcli::flags::*;
use toiletcli::flags;

use crate::docs::{deserialize_docs_json, download_docset_tar_gz, extract_docset_tar_gz};

use crate::common::ResultS;
use crate::common::{is_docs_json_exists, is_docset_downloaded, is_docset_in_docs, get_docset_path};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET, YELLOW};

fn show_download_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} download{RESET} [-f] <docset1> [docset2, ..]
    Download a docset. Available docsets can be displayed using `list`.

{GREEN}OPTIONS{RESET}
    -f, --force                 Overwrite downloaded docsets.
        --help                  Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn download<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>, {
    let mut flag_help;
    let mut flag_force;

    let mut flags = flags![
        flag_help: BoolFlag,  ["--help"],
        flag_force: BoolFlag, ["--force", "-f"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help || args.is_empty() { return show_download_help(); }

    if !is_docs_json_exists()? {
        return Err("`docs.json` does not exist. Please run `fetch` first".to_string());
    }

    let docs = deserialize_docs_json()?;
    let mut args_iter = args.iter();
    let mut success = 0;

    while let Some(docset) = args_iter.next() {
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

            println!("Extracting `{docset}` to `{}`...", get_docset_path(docset)?.display());
            extract_docset_tar_gz(docset)?;

            success += 1;
        }
    }

    if success > 1 {
        println!("{BOLD}{} items were successfully installed{RESET}.", success);
    } else if success == 1 {
        println!("{BOLD}Install successfully finished{RESET}.");
    }

    Ok(())
}
