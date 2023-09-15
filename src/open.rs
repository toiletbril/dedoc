use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
    deserialize_docs_json, is_docset_downloaded, is_docset_in_docs, print_page_from_docset,
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_open_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} open{RESET} [-i] <docset> <page>
    Print a page. Pages can be searched using `search`.

{GREEN}OPTIONS{RESET}
        --help             Display help message."
    );
    println!("{}", help);
    Ok(())
}

pub fn open<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;

    let mut flags = flags![
        flag_help: BoolFlag, ["--help"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_open_help(); }

    let mut args = args.iter();

    let docset = if let Some(docset_name) = args.next() {
        docset_name
    } else {
        return show_open_help();
    };

    if !is_docset_downloaded(docset)? {
        let message = if is_docset_in_docs(docset, &deserialize_docs_json()?) {
            format!("`{docset}` docset is not downloaded. Try using `download {docset}`.")
        } else {
            format!("`{docset}` does not exist. Try using `list` or `fetch`.")
        };
        return Err(message);
    }

    let mut query = args.fold(String::new(), |base, next| base + next + " ");
    query.pop(); // remove last space

    if query.is_empty() {
        return Err("No page specified. Try `open --help` for more information.".to_string());
    }

    print_page_from_docset(docset, &query)?;

    Ok(())
}
