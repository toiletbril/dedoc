#![cfg(debug_assertions)]

use super::*;

use std::fs::{File, remove_file, remove_dir_all};
use std::io::BufReader;
use std::sync::Once;
use std::vec::IntoIter;

use crate::common::{get_program_directory, get_docset_path};

use crate::fetch::fetch;
use crate::download::download;
use crate::search::{try_use_cache, search, SearchOptions};

pub fn create_args<'a>(args: &'a str) -> IntoIter<String> {
    args.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .into_iter()
}

pub fn remove_program_dir() -> ResultS {
    let program_directory = get_program_directory()?;
    remove_dir_all(&program_directory)
        .map_err(|err| format!("Could not remove `{}`: {err}", program_directory.display()))
}

static SETUP_INIT: Once = Once::new();

fn setup_test_directory() {
    SETUP_INIT.call_once(|| {
        dedoc_debug_println!("Removing program directory...");
        let _ = remove_program_dir();

        let empty_args = create_args("");

        let fetch_result = fetch(empty_args);
        assert!(fetch_result.is_ok());

        let download_args = create_args("backbone bower");

        let download_result = download(download_args);
        assert!(download_result.is_ok());

        let bower_path = get_docset_path(&"bower".to_string())
            .unwrap();
        let backbone_path = get_docset_path(&"backbone".to_string())
            .unwrap();

        assert!(bower_path.exists());
        assert!(backbone_path.exists());
    });

    assert!(SETUP_INIT.is_completed());

    dedoc_debug_println!("Directory has been set up.");
}

fn test_list_local() {
    let args = create_args("-l");

    dedoc_debug_println!("Running `ls -l`...");

    let list_result = list(args);
    assert!(list_result.is_ok());
}

fn test_search_should_use_cache() {
    let args = create_args("bower");

    let program_directory = get_program_directory().unwrap();

    let _ = remove_file(program_directory.join("search_cache_options.json"));

    dedoc_debug_println!("Running `search bower`");
    let search_result = search(args);
    assert!(search_result.is_ok());

    {
        let cache_options_path = program_directory.join("search_cache_options.json");
        let cache_options_file = File::open(&cache_options_path).unwrap();
        let cache_options_reader = BufReader::new(cache_options_file);

        let cached_search_options: SearchOptions = serde_json::from_reader(cache_options_reader).unwrap();

        assert!(try_use_cache(&cached_search_options).is_some());
    }

    dedoc_debug_println!("Search sucessfully created cache.");
}

fn test_search_case_insensitive() {
    let args_str = "backbone -i collection-at";
    let args = create_args(args_str);

    dedoc_debug_println!("Running `{args_str}`...");
    let search_result = search(args);
    assert!(search_result.is_ok());

    let args_str = "backbone -i collection-at -o 1";
    let args = create_args(args_str);

    dedoc_debug_println!("Running `{args_str}`...");
    let search_result = search(args);
    assert!(search_result.is_ok());
}

// Manual testing. I think this way is better than integration testing I came up with initially.
// If everything is looking cool, then it's we should be fine :3
pub fn test<Args>(_args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    setup_test_directory();

    test_list_local();
    test_search_should_use_cache();
    test_search_case_insensitive();

    dedoc_debug_println!("Tests completed.");

    Ok(())
}
