use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{
  create_program_directory, get_default_user_agent, get_flag_error, get_program_directory,
  is_docs_json_exists, is_docs_json_old, write_to_logfile,
};
use crate::common::{DocsEntry, ResultS};
use crate::common::{BOLD, DEFAULT_DOCS_JSON_LINK, GREEN, PROGRAM_NAME, RESET};

use ureq::get;

fn show_fetch_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} fetch{RESET} [-OPTIONS]
    Fetch latest `docs.json` which lists available languages and frameworks.

{GREEN}OPTIONS{RESET}
    -f, --force                     Force the download and overwrite
                                    `docs.json`.
        --help                      Display help message."
  );
  Ok(())
}

fn fetch_docs() -> Result<Vec<DocsEntry>, String>
{
  let response =
    get(DEFAULT_DOCS_JSON_LINK).set("User-Agent", &get_default_user_agent())
                               .set("Accept-Encoding", "gzip")
                               .call()
                               .map_err(|err| {
                                 format!("Could not fetch `{DEFAULT_DOCS_JSON_LINK}`: {err}")
                               })?;

  let body =
    response.into_string().map_err(|err| format!("Unable to read response body: {err}"))?;

  let docs: Vec<DocsEntry> =
    serde_json::from_str(body.as_str()).map_err(|err| {
      let result = write_to_logfile(format!("Error while parsing JSON body: {err}\n\n{body}"));
      let log_file_message = match result {
        Ok(path) => format!("Log file is saved at `{}`.", path.display()),
        Err(err) => format!("Unable to write log file: {err}."),
      };
      format!("Error while parsing JSON body: {err}. {log_file_message}")
    })?;

  Ok(docs)
}

fn serialize_and_overwrite_docs(path: PathBuf, docs: Vec<DocsEntry>) -> Result<(), String>
{
  let file =
    File::create(&path).map_err(|err| format!("Could not create `{}`: {err}", path.display()))?;

  let writer = BufWriter::new(file);

  serde_json::to_writer(writer, &docs).map_err(|err| {
                                        format!("Could not write `{}`: {err}", path.display())
                                      })?;

  Ok(())
}

pub(crate) fn fetch<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_force;
  let mut flag_help;

  let mut flags = flags![
    flag_force: BoolFlag, ["-f", "--force"],
    flag_help: BoolFlag,  ["--help"]
  ];

  parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;

  if flag_help {
    return show_fetch_help();
  }
  if !flag_force && is_docs_json_exists()? && !is_docs_json_old()? {
    return Err("It seems that your `docs.json` was updated less than a week \
                ago. Run `fetch --force` to ignore this warning."
                                                                 .to_string());
  }

  println!("Fetching `{DEFAULT_DOCS_JSON_LINK}`...");
  let docs = fetch_docs()?;

  let program_path = get_program_directory()?;
  if !program_path.exists() {
    create_program_directory()?;
  }
  let docs_json_path = program_path.join("docs.json");

  println!("Writing `{}`...", docs_json_path.display());
  serialize_and_overwrite_docs(docs_json_path, docs)?;
  println!("{BOLD}Fetching has successfully finished{RESET}.");

  Ok(())
}

#[cfg(test)]
mod tests
{
  use super::*;

  const TEST_STRING: &str = r#"
[
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
    }
]"#;

  #[test]
  fn test_parse_docs()
  {
    let json: Result<Vec<DocsEntry>, _> =
      serde_json::from_str(TEST_STRING).map_err(|err| err.to_string());

    assert_eq!(json.unwrap()[0].slug, "angular");
  }
}
