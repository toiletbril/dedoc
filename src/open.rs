use std::path::PathBuf;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
    deserialize_docs_json, is_docset_in_docs_or_print_warning, print_page_from_docset, print_docset_file
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_open_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} open{RESET} [-h] <docset> <page>
    Print a page. Pages can be searched using `search`.

{GREEN}OPTIONS{RESET}
    -h, --html                      Interpret arguments as a path to HTML file and translate it to markdown.
        --help                      Display help message."
    );
    Ok(())
}

pub fn open<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_html;

    let mut flags = flags![
        flag_help: BoolFlag, ["--help"],
        flag_html: BoolFlag, ["--html", "-h"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help || args.is_empty() { return show_open_help(); }

    if flag_html {
        let path = PathBuf::from(args.join(" "));
        return print_docset_file(path, None);
    }

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
