#![cfg(test)]

use super::*;

use std::fs::{File, remove_file};
use std::io::BufReader;
use std::sync::Once;

use crate::debug::*;
use crate::common::{get_program_directory, get_docset_path};

use crate::fetch::fetch;
use crate::download::download;
use crate::search::{try_use_cache, search, SearchOptions};

use toiletcli::common::overwrite_should_use_colors;

static SETUP_INIT: Once = Once::new();

fn setup_test_directory() {
    SETUP_INIT.call_once(|| {
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
}

#[test]
#[ignore]
fn test_search_should_use_cache() {
    setup_test_directory();

    let args = create_args("backbone");

    let program_directory = get_program_directory().unwrap();

    let _ = remove_file(program_directory.join("search_cache_options.json"));

    let search_result = search(args);
    assert!(search_result.is_ok());

    let cache_options_path = program_directory.join("search_cache_options.json");
    let cache_options_file = File::open(&cache_options_path).unwrap();
    let cache_options_reader = BufReader::new(cache_options_file);

    let cached_search_options: SearchOptions = serde_json::from_reader(cache_options_reader).unwrap();

    assert!(try_use_cache(&cached_search_options).is_some());
}

#[test]
#[ignore]
fn test_fetch_download_and_list_local() {
    setup_test_directory();

    unsafe {
        overwrite_should_use_colors(false);
        set_output_to_mock_output()
    }

    let args = create_args("-l");

    let list_result = list(args);
    assert!(list_result.is_ok());

    assert_eq!(&get_mock_output(), "backbone [downloaded], bower [downloaded]\n");
}
