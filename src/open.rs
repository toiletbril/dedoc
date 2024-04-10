use std::path::PathBuf;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
  deserialize_docs_json, get_flag_error, get_terminal_width,
  is_docs_json_exists, is_docset_in_docs_or_print_warning, print_docset_file,
  print_page_from_docset, split_to_item_and_fragment,
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};
use crate::print_warning;

fn show_open_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} open{RESET} [-hcn] <docset> <page>
    Print a page. Pages can be searched using `search`.

{GREEN}OPTIONS{RESET}
    -h, --html                      Interpret arguments as a path to HTML file
                                    and translate it to markdown.
    -c, --columns <number>          Make output N columns wide.
    -n, --line-numbers              Number output lines.
        --help                      Display help message."
  );
  Ok(())
}

pub(crate) fn open<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_html;
  let mut flag_columns;
  let mut flag_number_lines;
  let mut flag_help;

  let mut flags = flags![
    flag_html: BoolFlag,         ["-h", "--html"],
    flag_columns: StringFlag,    ["-c", "--columns"],
    flag_number_lines: BoolFlag, ["-n", "--line-numbers"],
    flag_help: BoolFlag,         ["--help"]
  ];

  let args =
    parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;
  if flag_help || args.is_empty() {
    return show_open_help();
  }

  let mut width = get_terminal_width();

  if let Ok(col_number) = flag_columns.parse::<usize>() {
    if col_number == 0 {
      width = 999;
    } else if col_number > 10 {
      width = col_number;
    }
  } else if !flag_columns.is_empty() {
    print_warning!("Invalid number of columns.");
  }

  if flag_html {
    let path = PathBuf::from(args.join(" "));
    print_docset_file(path, None, width, flag_number_lines)?;
    return Ok(());
  }

  if !is_docs_json_exists()? {
    return Err("The list of available documents has not yet been downloaded. \
                Please run `{PROGRAM_NAME} fetch` first.".to_string());
  }

  let mut args = args.into_iter();

  let docset = if let Some(docset_name) = args.next() {
    docset_name
  } else {
    return show_open_help();
  };

  if is_docset_in_docs_or_print_warning(&docset, &deserialize_docs_json()?) {
    let query = args.collect::<Vec<String>>().join(" ");
    if query.is_empty() {
      return Err("No page specified. Try `open --help` for more information."
                   .to_string());
    }

    let (item, fragment) = split_to_item_and_fragment(query)?;
    print_page_from_docset(&docset,
                           &item,
                           fragment.as_ref(),
                           width,
                           flag_number_lines)?;
  }

  Ok(())
}
