use std::{path::{Path, PathBuf}, io::{BufWriter, BufReader, Write, BufRead, Read}, fs::{create_dir, create_dir_all, remove_file, read_dir}};
use std::fs::File;

extern crate serde;
use flate2::bufread::GzDecoder;
use html2text::from_read;
use serde::{Serialize, Deserialize};

extern crate serde_json;

extern crate tinyquest;
use tar::Archive;
use tinyquest::get;

use crate::common::{VERSION, DEFAULT_DOCS_LINK, DEFAULT_USER_AGENT, DEFAULT_DOWNLOADS_LINK, get_docset_path, write_to_logfile, create_program_directory};
use crate::common::{get_program_directory, is_docs_json_exists};

#[inline(always)]
fn unknown_version() -> String {
    "unknown".to_string()
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Links {
    home: String,
    code: String
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Docs {
    #[serde(skip)]
    name: String,
    pub slug: String,
    #[serde(skip)]
    doctype: String,
    #[serde(skip)]
    links: Links,
    #[serde(default = "unknown_version")]
    pub version: String,
    #[serde(skip)]
    release: String,
    pub mtime: u64,
    db_size: usize,
    #[serde(skip)]
    attribution: String,
}

/* Example item:
{
    "name": "Angular",
    "slug": "angular",
    "type": "angular",
    "links": {
      "home": "https://google.com",
      "code": "https://google.com"
    },
    "version": "",
    "release": "16.1.3",
    "mtime": 1688411876,
    "db_size": 13128638,
    "attribution": "whatever"
} */

pub fn fetch_docs_json() -> Result<Vec<Docs>, String> {
    let user_agent = format!("{DEFAULT_USER_AGENT}/{VERSION}");

    let request = get(DEFAULT_DOCS_LINK, &user_agent)
        .and_then(|mut r| r.follow_redirects())
        .map_err(|err| format!("Could not GET `{DEFAULT_DOCS_LINK}`: {err:?}"))?;

    let body = String::from_utf8_lossy(request.body()).to_string();

    let docs: Vec<Docs> = serde_json::from_str(body.as_str())
        .map_err(|err| {
            let log_file_message = match write_to_logfile(format!("dasd")) {
                Ok(path) => format!("Log file is saved at {path:?}."),
                Err(err) => format!("Unable to write log file: {err}.")
            };
            format!("Serde error: {err}. {log_file_message}")
        })?;

    Ok(docs)
}

pub fn serialize_and_overwrite_docs_json(docs: Vec<Docs>) -> Result<(), String> {
    let program_path = get_program_directory()?;

    if !program_path.exists() {
        create_program_directory()?;
    }

    let docs_json_path = program_path.join("docs.json");
    let file = File::create(&docs_json_path)
        .map_err(|err| format!("{docs_json_path:?}: {err}"))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &docs)
        .map_err(|err| format!("Could not write {docs_json_path:?}: {err}"))?;

    Ok(())
}

pub fn deserealize_docs_json() -> Result<Vec<Docs>, String> {
    if !is_docs_json_exists()? {
        return Err("`docs.json` does not exist. Maybe run `fetch` first?".to_string());
    }

    let docs_json_path = get_program_directory()?.join("docs.json");
    let file = File::open(&docs_json_path)
        .map_err(|err| format!("Could not open {docs_json_path:?}: {err}"))?;

    let reader = BufReader::new(file);

    let docs = serde_json::from_reader(reader)
        .map_err(|err| format!("{err}. Maybe `docs.json` was modified?"))?;

    Ok(docs)
}

pub fn download_docset_tar_gz(docset_name: &String, docs: &Vec<Docs>) -> Result<(), String> {
    let user_agent = format!("{DEFAULT_USER_AGENT}/{VERSION}");

    for entry in docs.iter() {
        if docset_name == &entry.slug {
            let docsets_path = get_program_directory()?.join("docsets");
            let specific_docset_path = docsets_path.join(&docset_name);

            if !specific_docset_path.exists() {
                create_dir_all(&specific_docset_path)
                    .map_err(|err| format!("Cannot create `{docset_name}` directory: {err}"))?;
            }

            let tar_gz_path = specific_docset_path.join(docset_name).with_extension("tar.gz");

            let mut file = File::create(&tar_gz_path)
                .map_err(|err| format!("Could not create `{tar_gz_path:?}`: {err}"))?;

            let download_link = format!("{DEFAULT_DOWNLOADS_LINK}/{docset_name}.tar.gz");

            let request = get(&download_link, user_agent.as_str())
                .and_then(|mut r| r.follow_redirects())
                .map_err(|err| format!("Could not GET {download_link}: {err:?}"))?;

            let body = request.body();

            file.write_all(body)
                .map_err(|err| format!("{err:?}"))?;
        }
    }

    Ok(())
}

pub fn extract_docset_tar_gz(docset_name: &String) -> Result<(), String> {
    let docset_path = get_docset_path(docset_name)?;

    if !docset_path.exists() {
        create_dir_all(&docset_path)
            .map_err(|err| format!("Cannot create `{docset_name}` directory: {err}"))?;
    }

    let tar_gz_path = docset_path.join(docset_name).with_extension("tar.gz");

    let tar_gz_file = File::open(&tar_gz_path)
        .map_err(|err| format!("Could not open {tar_gz_path:?}: {err}"))?;

    let reader = BufReader::new(tar_gz_file);
    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);

    archive.unpack(docset_path)
        .map_err(|err| format!("Could not unpack {tar_gz_path:?}: {err}"))?;

    remove_file(&tar_gz_path)
        .map_err(|err| format!("Could not remove {tar_gz_path:?}: {err}"))?;

    Ok(())
}

pub fn search_docset_in_filenames(
    docset_name: &String,
    query: &String,
    case_insensitive: bool
) -> Result<Vec<PathBuf>, String> {
    let docset_path = get_docset_path(docset_name)?;

    let _query = if case_insensitive {
        query.to_lowercase()
    } else {
        query.to_owned()
    };

    fn visit_dir_with_query(path: &PathBuf, _query: &String, case_insensitive: bool) -> Result<Vec<PathBuf>, String> {
        let mut internal_paths = vec![];

        let dir = read_dir(&path)
            .map_err(|err| format!("Could not read directory {path:?}: {err}"))?;

        for entry in dir {
            let entry = entry
                .map_err(|err| format!("Could not read file: {err}"))?;

            let _file_name = entry.file_name();

            let file_type = entry.file_type()
                .map_err(|err| format!("Could not read file type of {_file_name:?}: {err}"))?;

            if file_type.is_dir() {
                let mut visited = visit_dir_with_query(&entry.path(), &_query, case_insensitive)?;
                internal_paths.append(&mut visited);
            }

            let mut file_name = _file_name.to_string_lossy().to_string();

            if file_name.rfind(".html").is_none() {
                continue;
            }

            if case_insensitive {
                file_name.make_ascii_lowercase();
            }

            if file_name.find(_query).is_some() {
                internal_paths.push(entry.path());
            }
        }
        Ok(internal_paths)
    }

    let result = visit_dir_with_query(&docset_path, &_query, case_insensitive)?;

    Ok(result)
}

type ExactMatches = Vec<PathBuf>;
type VagueMatches = Vec<PathBuf>;

// @@@
pub fn search_docset_thoroughly(
    docset_name: &String,
    query: &String,
    case_insensitive: bool
) -> Result<(ExactMatches, VagueMatches), String> {
    let docset_path = get_docset_path(docset_name)?;

    let _query = if case_insensitive {
        query.to_lowercase()
    } else {
        query.to_owned()
    };

    fn visit_dir_with_query(
        path: &PathBuf,
        _query: &String,
        case_insensitive: bool
     ) -> Result<(ExactMatches, VagueMatches), String> {
        let mut exact_paths = vec![];
        let mut vague_paths = vec![];

        let dir = read_dir(&path)
            .map_err(|err| format!("Could not read directory {path:?}: {err}"))?;

        for entry in dir {
            let entry = entry
                .map_err(|err| format!("Could not read file: {err}"))?;

            let _file_name = entry.file_name();

            let file_type = entry.file_type()
                .map_err(|err| format!("Could not read file type of {_file_name:?}: {err}"))?;

            if file_type.is_dir() {
                let (mut exact, mut vague) = visit_dir_with_query(&entry.path(), &_query, case_insensitive)?;
                exact_paths.append(&mut exact);
                vague_paths.append(&mut vague);
            }

            let mut file_name = _file_name.to_string_lossy().to_string();

            if file_name.rfind(".html").is_none() {
                continue;
            }

            if case_insensitive {
                file_name.make_ascii_lowercase();
            }

            let file_path = entry.path();

            if file_name.find(_query).is_some() {
                exact_paths.push(file_path);
            } else {
                let file = File::open(&file_path)
                    .map_err(|err| format!("Could not open {file_path:?}: {err}"))?;
                let mut reader = BufReader::new(file);
                let mut string_buffer = String::new();

                while let Ok(size) = reader.read_line(&mut string_buffer) {
                    if size == 0 {
                        break;
                    }

                    if string_buffer.find(_query).is_some() {
                        vague_paths.push(entry.path());
                        break;
                    }

                    string_buffer.clear();
                }
            }
        }
        Ok((exact_paths, vague_paths))
    }

    let result = visit_dir_with_query(&docset_path, &_query, case_insensitive)?;

    Ok(result)
}

pub fn print_page_from_docset(docset_name: &String, page: &String) -> Result<(), String> {
    let docset_path = get_docset_path(docset_name)?;

    let mut file_path = docset_path.join(page.to_owned() + ".html");

    println!("path: {file_path:?}");

    if !file_path.is_file() {
        let message = format!("No page matching `{page}`. Did you specify the name from `search` correctly?");
        return Err(message);
    }

    let file = File::open(&file_path)
        .map_err(|err| format!("Could not open {file_path:?}: {err}"))?;
    let mut reader = BufReader::new(file);

    let page = from_read(reader, 80);

    println!("{}", page.trim());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = "\
[
    {
        \"name\": \"Angular\",
        \"slug\": \"angular\",
        \"type\": \"angular\",
        \"links\": {
            \"home\": \"https://google.com\",
            \"code\": \"https://google.com\"
        },
        \"version\": \"\",
        \"release\": \"16.1.3\",
        \"mtime\": 1688411876,
        \"db_size\": 13128638,
        \"attribution\": \"whatever\"
    }
]";

    #[test]
    fn test_parse_docs() {
        let json: Result<Vec<Docs>, _> = serde_json::from_str(TEST_STRING)
            .map_err(|err| err.to_string());

        assert_eq!(json.unwrap()[0].slug, "angular");
    }
}
