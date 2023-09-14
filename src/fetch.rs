use toiletcli::flags::*;
use toiletcli::flags;

use crate::docs::{fetch_docs_json, serialize_and_overwrite_docs_json};
use crate::common::{is_docs_json_exists, is_docs_json_old};
use crate::common::{BOLD, DEFAULT_DOCS_LINK, GREEN, PROGRAM_NAME, RESET, YELLOW};

fn show_fetch_help() -> Result<(), String> {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} fetch{RESET} [-f]
    Fetch latest `docs.json` which lists available languages and frameworks.

{GREEN}OPTIONS{RESET}
    -f, --force                 Update even if `docs.json` is recent.
        --help                  Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn fetch<Args>(mut args: Args) -> Result<(), String>
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_force;

    let mut flags = flags![
        flag_help: BoolFlag,  ["--help"],
        flag_force: BoolFlag, ["--force", "-f"]
    ];

    parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_fetch_help(); }

    if !flag_force && is_docs_json_exists()? && !is_docs_json_old()? {
        let message = format!(
            "\
            {YELLOW}WARNING{RESET}: It seems that your `docs.json` was updated less than a week ago. \
            Run `fetch --force` to ignore this warning."
        );
        println!("{}", message);
        return Ok(());
    }

    println!("Fetching `{DEFAULT_DOCS_LINK}`...");
    let docs = fetch_docs_json()?;

    println!("Writing `docs.json`...");
    serialize_and_overwrite_docs_json(docs)?;

    println!("Successfully updated `docs.json`.");

    Ok(())
}
