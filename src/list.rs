use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{is_docset_downloaded, make_sure_docset_is_in_docs, ResultS};
use crate::common::{
  deserialize_docs_json, get_flag_error, get_local_docsets, is_docs_json_exists,
};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};
use crate::print_warning;

fn show_list_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} list{RESET} [-lans]
    Show available docsets.

{GREEN}OPTIONS{RESET}
    -l, --local                     Only show local docsets.
    -o, --non-local                 Only show docsets that haven't been
                                    downloaded.
    -a, --all                       Show all version-specific docsets.
    -n, --newlines                  Print each docset on a separate line.
    -d, --no-labels                 Don't print `[downloaded]` labels.
        --porcelain                 Same as -nd.
    -s, --search <query>            Filter docsets based on a query.
    -e, --exists <docset>           Error out if docset does not exist.
        --help                      Display help message."
  );
  Ok(())
}

pub(crate) fn list<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_all;
  let mut flag_local;
  let mut flag_nonlocal;
  let mut flag_newlines;
  let mut flag_labels;
  let mut flag_search;
  let mut flag_porcelain;
  let mut flag_exists;
  let mut flag_help;

  let mut flags = flags![
    flag_all: BoolFlag,       ["-a", "--all"],
    flag_local: BoolFlag,     ["-l", "--local"],
    flag_nonlocal: BoolFlag,  ["-o", "--non-local"],
    flag_newlines: BoolFlag,  ["-n", "--newlines"],
    flag_labels: BoolFlag,    ["-d", "--no-labels"],
    flag_search: StringFlag,  ["-s", "--search"],
    flag_porcelain: BoolFlag, ["--porcelain"],
    flag_exists: StringFlag,  ["-e", "--exists"],
    flag_help: BoolFlag,      ["--help"]
  ];

  let args = parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;
  if flag_help {
    return show_list_help();
  }
  if !args.is_empty() {
    print_warning!("Arguments were not used.");
  }

  if !is_docs_json_exists()? {
    return Err(format!(
      "The list of available documents has not yet been downloaded. \
       Please run `{PROGRAM_NAME} fetch` first."
    ));
  }

  if flag_porcelain {
    flag_labels = true;
    flag_newlines = true;
  }

  let local_docsets = get_local_docsets()?;
  let should_filter = !flag_search.is_empty();
  let separator = if flag_newlines { "\n" } else { ", " };

  // Show everything when searching.
  if should_filter {
    flag_all = true;
  }

  if flag_local && flag_nonlocal {
    return Err("Both -o and -l are enabled. Please make a final decision.".to_string());
  }

  if !flag_exists.is_empty() {
    let docs = deserialize_docs_json()?;

    make_sure_docset_is_in_docs(&flag_exists, &docs)?;

    if flag_local {
      if is_docset_downloaded(&flag_exists)? {
        return Ok(())
      }
      return Err(format!("Docset `{flag_exists}` is not downloaded."));
    }

    return Ok(());
  }

  let mut first_result = true;

  if flag_local {
    for ref entry in local_docsets {
      if should_filter && !entry.contains(&flag_search) {
        continue;
      }
      if !first_result {
        print!("{}", separator);
      }
      print!("{GREEN}{}", entry);
      if !flag_labels {
        print!(" [downloaded]");
      }
      print!("{RESET}");
      first_result = false;
    }
    if !first_result {
      println!();
    } else {
      return Err("Nothing to do.".to_string());
    }

    return Ok(());
  }

  let docs = deserialize_docs_json()?;
  let docs_names = docs.iter().map(|entry| entry.slug.to_string()).collect::<Vec<String>>();

  for ref entry in docs_names {
    if should_filter && !entry.contains(&flag_search) {
      continue;
    }
    // slug has ~ if it's version-specific
    if !flag_local && !flag_all && entry.contains('~') {
      continue;
    }
    if local_docsets.contains(entry) {
      if flag_nonlocal {
        continue;
      }
      if !first_result {
        print!("{}", separator);
      }
      print!("{GREEN}{}", entry);
      if !flag_labels {
        print!(" [downloaded]");
      }
      print!("{RESET}");
    } else {
      if !first_result {
        print!("{}", separator);
      }
      print!("{}", entry);
    }
    first_result = false;
  }

  // A final newline only if something was found.
  if !first_result {
    println!();
  }

  Ok(())
}
