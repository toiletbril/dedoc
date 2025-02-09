#![allow(unused)]

use std::fs::{remove_dir_all, remove_file, File};
use std::io::BufReader;
use std::vec::IntoIter;

use crate::common::ResultS;
use crate::common::{get_flag_error, get_program_directory};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RED, RESET};
use crate::debug_println;

use crate::download::download;
use crate::fetch::fetch;
use crate::list::list;
use crate::open::open;
use crate::remove::remove;
use crate::search::{search, try_use_cache, SearchContext};

use toiletcli::flags;
use toiletcli::flags::{parse_flags, FlagType};

fn show_test_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} test{RESET} [-f] <docset> <page>
    Run the testing suite.

{GREEN}OPTIONS{RESET}
    -f, --force                     Run all tests, including `download` and
                                    `fetch`.
        --help                      Display help message."
  );
  Ok(())
}

fn reset_state_and_cache()
{
  debug_println!("Removing cache...");
  let program_directory = get_program_directory().unwrap();

  let _ = remove_file(program_directory.join("docs.json"));
  let _ = remove_file(program_directory.join("search_cache_options.json"));
  let _ = remove_file(program_directory.join("search_cache.json"));
}

fn create_args(args: &str) -> IntoIter<String>
{
  args.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>().into_iter()
}

fn run_with_args<O>(command: fn(IntoIter<String>) -> Result<O, String>,
                    args_str: &str,
                    should_do: &str)
                    -> bool
{
  debug_println!("Running with args: `{args_str}`, should {should_do}");

  if let Err(err) = command(create_args(args_str)) {
    debug_println!("{BOLD}*** Test failed with: {err} ***");
    false
  } else {
    true
  }
}

fn test_search_should_use_cache(args: &str)
{
  let program_directory = get_program_directory().unwrap();
  let _ = remove_file(program_directory.join("search_cache_options.json"));

  run_with_args(search, args, "print search results");

  let cache_options_path = program_directory.join("search_cache_options.json");
  let cache_options_file = File::open(cache_options_path).unwrap();
  let cache_options_reader = BufReader::new(cache_options_file);

  let cached_search_options: SearchContext = serde_json::from_reader(cache_options_reader).unwrap();

  assert!(try_use_cache(&cached_search_options).is_some());

  debug_println!("Search sucessfully created cache.");
}

// Manual testing. I think this way is better than integration testing I came up
// with initially. If everything is looking cool, then it's we should be fine :3
pub(crate) fn debug_test<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_full;
  let mut flag_help;

  let mut flags = flags![
    flag_full: BoolFlag, ["-f", "--full"],
    flag_help: BoolFlag, ["--help"]
  ];

  let args = parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;

  if flag_help {
    return show_test_help();
  }
  if flag_full {
    reset_state_and_cache();

    run_with_args(fetch, "", "fetch docs.json");
    run_with_args(fetch, "", "show a fetch warning");

    run_with_args(remove, "backbone bower", "remove backbone and bower if they exist");
    run_with_args(download, "backbone bower", "download docsets");
  } else {
    debug_println!("Skipping `fetch` and `download`. Use `-f` flag to avoid skipping.");
  }

  run_with_args(download, "erl", "suggest three erlang versions");
  run_with_args(download, "win", "suggest tailwind");

  run_with_args(list, "-l", "list bower and backbone");

  test_search_should_use_cache("backbone");

  debug_println!("Performing search tests...");

  let search_results =
    [run_with_args(search, "backbone -o 1", "open first page"),
     run_with_args(search, "backbone -o 100", "open 100th page"),
     run_with_args(search, "backbone -i collection-at", "list search results in correct casing"),
     run_with_args(search, "backbone -i collection-at -o 1", "open first page"),
     run_with_args(search, "backbone -p map", "list precise search results"),
     run_with_args(search, "backbone -pi underscore", "list underscore.js with right case"),
     // This fails, because for some reason html2text fails to find fragments
     // that are inside of <li> tags :(
     run_with_args(search, "backbone -o 150", "show model-values"),
     run_with_args(search, "bower", "list bower results"),
     run_with_args(search, "bower -o 18", "show update"),
     run_with_args(search, "bower -o 3", "show cache"),
     run_with_args(search, "bower hahaha nothing", "should result in no matches")];

  let times_search_failed = search_results.iter().filter(|res| !*res).count();
  debug_println!("Search tests completed, {times_search_failed} tests failed.");

  Ok(())
}
