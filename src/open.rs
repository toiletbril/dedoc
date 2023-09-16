use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
    deserialize_docs_json, is_docset_in_docs_or_print_warning, print_page_from_docset,
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_open_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} open{RESET} [-i] <docset> <page>
    Print a page. Pages can be searched using `search`.

{GREEN}OPTIONS{RESET}
        --help                      Display help message."
    );
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

    let mut args = args.into_iter();

    let docset = if let Some(docset_name) = args.next() {
        docset_name
    } else {
        return show_open_help();
    };

    let docs = deserialize_docs_json()?;

    if is_docset_in_docs_or_print_warning(&docset, &docs) {
        let query = args.collect::<Vec<String>>().join(" ");

        if query.is_empty() {
            return Err("No page specified. Try `open --help` for more information.".to_string());
        }

        print_page_from_docset(&docset, &query)?;
    }

    Ok(())
}
