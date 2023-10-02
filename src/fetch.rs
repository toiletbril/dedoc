use std::fs::File;
use std::path::PathBuf;
use std::io::BufWriter;

use attohttpc::get;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{Docs, ResultS};
use crate::common::{
    create_program_directory, get_program_directory, is_docs_json_exists, is_docs_json_old,
    write_to_logfile,
};
use crate::common::{
    BOLD, DEFAULT_DOCS_JSON_LINK, DEFAULT_USER_AGENT, GREEN, PROGRAM_NAME, RESET, VERSION, YELLOW,
};

fn show_fetch_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} fetch{RESET} [-f]
    Fetch latest `docs.json` which lists available languages and frameworks.

{GREEN}OPTIONS{RESET}
    -f, --force                     Force the download and overwrite `docs.json`.
        --help                      Display help message."
    );
    Ok(())
}

fn fetch_docs() -> Result<Vec<Docs>, String> {
    let user_agent = format!("{DEFAULT_USER_AGENT}/{VERSION}");

    let response = get(DEFAULT_DOCS_JSON_LINK)
        .header_append("user-agent", user_agent)
        .send()
        .map_err(|err| format!("Could not GET `{DEFAULT_DOCS_JSON_LINK}`: {err:?}"))?;

    let body = response
        .text()
        .map_err(|err| format!("Unable to read response body: {err}"))?;

    let docs: Vec<Docs> = serde_json::from_str(body.as_str()).map_err(|err| {
        let result = write_to_logfile(format!("Error while parsing JSON body: {err}\n\n{body}"));
        let log_file_message = match result {
                Ok(path) => format!("Log file is saved at `{}`.", path.display()),
                Err(err) => format!("Unable to write log file: {err}."),
        };
        format!("Error while parsing JSON body: {err}. {log_file_message}")
    })?;

    Ok(docs)
}

fn serialize_and_overwrite_docs(path: PathBuf, docs: Vec<Docs>) -> Result<(), String> {
    let file = File::create(&path)
        .map_err(|err| format!("`{}`: {err}", path.display()))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &docs)
        .map_err(|err| format!("Could not write `{}`: {err}", path.display()))?;

    Ok(())
}

pub fn fetch<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_force;

    let mut flags = flags![
        flag_help: BoolFlag,  ["--help"],
        flag_force: BoolFlag, ["--force", "-f"]
    ];

    parse_flags(&mut args, &mut flags)?;
    if flag_help { return show_fetch_help(); }

    if !flag_force && is_docs_json_exists()? && !is_docs_json_old()? {
        let message = format!(
            "\
{YELLOW}WARNING{RESET}: It seems that your `docs.json` was updated less than a week ago. \
Run `fetch --force` to ignore this warning."
        );
        println!("{}", message);
        return Ok(());
    }

    println!("Fetching `{DEFAULT_DOCS_JSON_LINK}`...");
    let docs = fetch_docs()?;

    let program_path = get_program_directory()?;
    let docs_json_path = program_path.join("docs.json");

    if !program_path.exists() {
        create_program_directory()?;
    }

    println!("Writing `{}`...", docs_json_path.display());
    serialize_and_overwrite_docs(docs_json_path, docs)?;

    println!("{BOLD}Fetching has successfully finished{RESET}.");

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
        let json: Result<Vec<Docs>, _> =
            serde_json::from_str(TEST_STRING).map_err(|err| err.to_string());

        assert_eq!(json.unwrap()[0].slug, "angular");
    }
}
