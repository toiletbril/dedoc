#![cfg(debug_assertions)]

use std::fs::{File, remove_file, remove_dir_all};
use std::io::BufReader;
use std::vec::IntoIter;

use crate::common::ResultS;
use crate::common::get_program_directory;
use crate::common::{RESET, RED};

use crate::dedoc_debug_println;

use crate::list::list;
use crate::fetch::fetch;
use crate::remove::remove;
use crate::download::download;
use crate::search::{try_use_cache, search, SearchOptions};

pub fn create_args<'a>(args: &'a str) -> IntoIter<String> {
    args.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .into_iter()
}

pub fn remove_program_dir() -> ResultS {
    dedoc_debug_println!("Removing program directory...");

    let program_directory = get_program_directory()?;

    remove_dir_all(&program_directory)
        .map_err(|err| format!("Could not remove `{}`: {err}", program_directory.display()))
}

fn run_with_args(command: fn(IntoIter<String>) -> ResultS, args_str: &str, should_do: &str) {
    let args = create_args(args_str);
    dedoc_debug_println!("Running with args: `{args_str}`, should {should_do}");

    let command_result = command(args);
    assert!(command_result.is_ok());
}

fn test_search_should_use_cache() {
    let program_directory = get_program_directory().unwrap();
    let _ = remove_file(program_directory.join("search_cache_options.json"));

    run_with_args(search, "bower", "print search results");

    {
        let cache_options_path = program_directory.join("search_cache_options.json");
        let cache_options_file = File::open(&cache_options_path).unwrap();
        let cache_options_reader = BufReader::new(cache_options_file);

        let cached_search_options: SearchOptions = serde_json::from_reader(cache_options_reader).unwrap();

        assert!(try_use_cache(&cached_search_options).is_some());
    }

    dedoc_debug_println!("Search sucessfully created cache.");
}

// Manual testing. I think this way is better than integration testing I came up with initially.
// If everything is looking cool, then it's we should be fine :3
pub fn test<Args>(_args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let _ = remove_program_dir();

    run_with_args(fetch, "", "fetch docs.json");
    run_with_args(download, "backbone bower", "download docsets");

    test_search_should_use_cache();

    run_with_args(list, "-l", "list bower and backbone");

    run_with_args(search, "backbone -i collection-at", "list search results in correct casing");
    run_with_args(search, "backbone -i collection-at -o 1", "open first page");
    run_with_args(search, "backbone -p map", "list precise search results");

    run_with_args(fetch, "", "show a fetch warning");
    run_with_args(download, "win", "suggest tailwind");
    run_with_args(download, "erl", "suggest three erlang versions");
    run_with_args(remove, "--purge-all", "delete docsets");

    dedoc_debug_println!("Tests completed.");

    Ok(())
}
