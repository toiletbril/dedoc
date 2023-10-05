#![allow(unused)]
#![cfg(debug_assertions)]

use std::fs::{File, remove_file, remove_dir_all};
use std::io::BufReader;
use std::vec::IntoIter;

use crate::common::ResultS;
use crate::common::get_program_directory;
use crate::common::{RED, GREEN, BOLD, RESET, PROGRAM_NAME};
use crate::debug_println;

use crate::open::open;
use crate::remove::remove;
use crate::download::download;
use crate::search::{SearchOptions, try_use_cache, search};
use crate::list::list;
use crate::fetch::fetch;

use toiletcli::flags::{FlagType, parse_flags};
use toiletcli::flags;

fn show_test_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} test{RESET} [-f] <docset> <page>
    Run the testing suite.

{GREEN}OPTIONS{RESET}
    -f, --force                     Run all tests, including `download` and `fetch`.
        --help                      Display help message."
    );
    Ok(())
}

pub fn create_args<'a>(args: &'a str) -> IntoIter<String> {
    args.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .into_iter()
}

pub fn reset_state_and_cache() {
    debug_println!("Removing cache...");

    let program_directory = get_program_directory().unwrap();

    let _ = remove_file(program_directory.join("docs.json"));
    let _ = remove_file(program_directory.join("search_cache_options.json"));
    let _ = remove_file(program_directory.join("search_cache.json"));
}

fn run_with_args<O, E>(
    command: fn(IntoIter<String>) -> Result<O, E>,
    args_str: &str, should_do: &str
) -> Result<O,E> {
    let args = create_args(args_str);
    debug_println!("Running with args: `{args_str}`, should {should_do}");

    let command_result = command(args);
    assert!(command_result.is_ok());

    command_result
}

fn test_search_should_use_cache(args: &str) {
    let program_directory = get_program_directory().unwrap();
    let _ = remove_file(program_directory.join("search_cache_options.json"));

    run_with_args(search, args, "print search results");

    {
        let cache_options_path = program_directory.join("search_cache_options.json");
        let cache_options_file = File::open(&cache_options_path).unwrap();
        let cache_options_reader = BufReader::new(cache_options_file);

        let cached_search_options: SearchOptions = serde_json::from_reader(cache_options_reader).unwrap();

        assert!(try_use_cache(&cached_search_options).is_some());
    }

    debug_println!("Search sucessfully created cache.");
}

// Manual testing. I think this way is better than integration testing I came up with initially.
// If everything is looking cool, then it's we should be fine :3
pub fn test<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_full;

    let mut flags = flags![
        flag_help: BoolFlag, ["--help"],
        flag_full: BoolFlag, ["-f", "--full"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_test_help(); }

    if flag_full {
        reset_state_and_cache();

        run_with_args(fetch, "", "fetch docs.json");
        run_with_args(fetch, "", "show a fetch warning");

        run_with_args(remove, "backbone bower", "remove backbone and bower if they exist");
        run_with_args(download, "backbone bower", "download docsets");
    } else {
        debug_println!("Skipping `fetch` and `download`. Use `-f` flag to avoid skipping.");
    }

    run_with_args(download, "win", "suggest tailwind");
    run_with_args(download, "erl", "suggest three erlang versions");

    run_with_args(list, "-l", "list bower and backbone");

    test_search_should_use_cache("backbone");

    run_with_args(search, "backbone -o 1", "open first page");
    run_with_args(search, "backbone -o 100", "open 100th page");
    run_with_args(search, "backbone -i collection-at", "list search results in correct casing");
    run_with_args(search, "backbone -i collection-at -o 1", "open first page");
    run_with_args(search, "backbone -p map", "list precise search results");

    run_with_args(search, "bower", "should list bower results");
    run_with_args(search, "bower -o 18", "should open matching result");
    run_with_args(search, "bower -o 3", "should open matching result");
    run_with_args(search, "bower hahaha nothing", "should result in no matches");

    debug_println!("Tests completed.");

    Ok(())
}
