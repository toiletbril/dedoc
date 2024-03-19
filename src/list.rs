use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;
use crate::common::{
  deserialize_docs_json, get_flag_error, get_local_docsets, is_docs_json_exists,
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};

fn show_list_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} list{RESET} [-lans]
    Show available docsets.

{GREEN}OPTIONS{RESET}
    -l, --local                     Show only local docsets.
    -a, --all                       Show all version-specific docsets.
    -n, --newlines                  Print each docset on a separate line.
    -s, --search <query>            Filter docsets based on a query.
        --help                      Display help message."
  );
  Ok(())
}

pub(crate) fn list<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_all;
  let mut flag_local;
  let mut flag_newlines;
  let mut flag_search;
  let mut flag_help;

  let mut flags = flags![
    flag_all: BoolFlag,      ["-a", "--all"],
    flag_local: BoolFlag,    ["-l", "--local"],
    flag_newlines: BoolFlag, ["-n", "--newlines"],
    flag_search: StringFlag, ["-s", "--search"],
    flag_help: BoolFlag,     ["--help"]
  ];

  parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;
  if flag_help
  {
    return show_list_help();
  }

  if !is_docs_json_exists()?
  {
    return Err("The list of available documents has not yet been downloaded. \
                Please run `{PROGRAM_NAME} fetch` first.".to_string());
  }

  let mut first_result = true;

  let local_docsets = get_local_docsets()?;
  let should_filter = !flag_search.is_empty();
  let separator = if flag_newlines { "\n" } else { ", " };

  // Show everything when searching :3c
  if should_filter
  {
    flag_all = true;
  }

  if flag_local
  {
    for ref entry in local_docsets
    {
      if should_filter && !entry.contains(&flag_search)
      {
        continue;
      }
      if !first_result
      {
        print!("{}", separator);
      }
      print!("{GREEN}{} [downloaded]{RESET}", entry);
      first_result = false;
    }
    if !first_result
    {
      println!();
    }

    return Ok(());
  }

  let docs = deserialize_docs_json()?;
  let docs_names =
    docs.iter().map(|entry| entry.slug.to_string()).collect::<Vec<String>>();

  for ref entry in docs_names
  {
    if should_filter && !entry.contains(&flag_search)
    {
      continue;
    }
    // slug has ~ if it's version-specific
    if !flag_local && !flag_all && entry.contains('~')
    {
      continue;
    }
    if !first_result
    {
      print!("{}", separator);
    }
    if local_docsets.contains(entry)
    {
      print!("{GREEN}{} [downloaded]{RESET}", entry);
    }
    else
    {
      print!("{}", entry);
    }
    first_result = false;
  }
  // Print a final newline only if something was found
  if !first_result
  {
    println!();
  }

  Ok(())
}
