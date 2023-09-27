use std::fs::{create_dir_all, remove_file, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use attohttpc::get;

use serde::Deserializer;
use serde::de::{Visitor, Error, MapAccess};

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{Docs, ResultS};
use crate::common::{
    deserialize_docs_json, get_docset_path, is_docs_json_exists, is_docset_downloaded,
    is_docset_in_docs_or_print_warning
};
use crate::common::{
    BOLD, DEFAULT_DB_JSON_LINK, DEFAULT_USER_AGENT, GREEN, PROGRAM_NAME, RESET, VERSION, YELLOW,
};

fn show_download_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} download{RESET} [-f] <docset1> [docset2, ..]
    Download a docset. Available docsets can be displayed using `list`.

{GREEN}OPTIONS{RESET}
    -f, --force                     Force the download and overwrite files.
        --help                      Display help message."
    );
    Ok(())
}

fn download_db_and_index_json_with_progress(
    docset_name: &String,
    docs: &Vec<Docs>,
) -> ResultS {
    let user_agent = format!("{DEFAULT_USER_AGENT}/{VERSION}");

    for entry in docs.iter() {
        if docset_name == &entry.slug {
            let docset_path = get_docset_path(docset_name)?;

            if !docset_path.exists() {
                create_dir_all(&docset_path)
                    .map_err(|err| format!("Cannot create `{}` directory: {err}", docset_path.display()))?;
            }

            for (file_name, i) in [("db.json", 1), ("index.json", 2)] {
                let file_path = docset_path
                    .join(file_name);

                let file = File::create(&file_path)
                    .map_err(|err| format!("Could not create `{}`: {err}", file_path.display()))?;

                let download_link = format!("{DEFAULT_DB_JSON_LINK}/{docset_name}/{}?{}", file_name, entry.mtime);

                let response = get(&download_link)
                    .header_append("user-agent", &user_agent)
                    .send()
                    .map_err(|err| format!("Could not GET {download_link}: {err}"))?;

                let mut file_writer = BufWriter::new(file);
                let mut response_reader = BufReader::new(response);

                let mut buffer = [0; 1024 * 4];
                let mut file_size = 0;

                while let Ok(size) = response_reader.read(&mut buffer) {
                    if size == 0 {
                        break;
                    }

                    file_writer
                        .write(&buffer[..size])
                        .map_err(|err| format!("Could not download file: {err}"))?;

                    file_size += size;

                    print!("\rReceived {file_size} bytes, file {i} of 2...");
                }
                println!();
            }
        }
    }

    Ok(())
}

// Remove id="..." and class="..." attributes from HTML tags to reduce size.
fn sanitize_html_line<'a>(input: String) -> String {
    enum State {
        Default,
        InTag,
        InKey,
        InValue,
    }

    let length = input.len();

    let mut output = String::new();
    let mut state = State::Default;
    let mut position = 0;

    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        match state {
            State::Default => {
                if ch == '<' {
                    state = State::InTag;
                }
                output.push(ch);
            }
            State::InTag => {
                let bytes = input.as_bytes();
                match ch {
                    'i' if position + 4 < length && bytes[position..position + 4] == *b"id=\""
                        => {
                        state = State::InKey;
                    }
                    'c' if position + 7 < length && bytes[position..position + 7] == *b"class=\"" => {
                        state = State::InKey;
                    }
                    '>' => {
                        state = State::Default;
                        output.push(ch);
                    }
                    _ => output.push(ch)
                }
            }
            State::InKey => {
                if ch == '\"' {
                    state = State::InValue;
                }
            }
            State::InValue => {
                if ch == '\"' {
                    state = State::InTag;
                }
            }
        }
        position += ch.len_utf8();
    }

    output
}

fn build_docset_from_map_with_progress<'de, M>(docset_name: &String, mut map: M) -> ResultS
where
    M: MapAccess<'de>,
{
    #[inline]
    #[cfg(target_family = "windows")]
    fn sanitize_filename_for_windows(filename: String) -> String {
        const FORBIDDEN_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];
        filename
            .chars()
            .map(|c| if FORBIDDEN_CHARS.contains(&c) { '_' } else { c })
            .collect::<String>()
    }

    let docset_path = get_docset_path(docset_name)?;
    let mut unpacked_amount = 1;

    while let Some((file_path, contents)) = map.next_entry::<String, String>()
        .map_err(|err| err.to_string())?
    {
        #[cfg(target_family = "windows")]
        let file_path = sanitize_filename_for_windows(file_path);
        let file_path = PathBuf::from(file_path);

        if let Some(parent) = file_path.parent() {
            create_dir_all(docset_path.join(parent))
                .map_err(|err| format!("Could not create `{}`: {err}", parent.display()))?;
        }

        let mut file_name_html = file_path.as_os_str().to_owned();
        file_name_html.push(".html");

        let file_path = docset_path.join(&file_name_html);

        let file = File::create(&file_path)
            .map_err(|err| format!("Could not create `{}`: {err}", file_path.display()))?;
        let mut writer = BufWriter::new(file);

        let sanitized_contents = sanitize_html_line(contents);

        writer.write_all(sanitized_contents.trim().as_bytes())
            .map_err(|err| format!("Could not write to `{}`: {err}", file_path.display()))?;

        print!("Unpacked {unpacked_amount} files...\r");

        unpacked_amount += 1;
    }
    println!();

    Ok(())
}

struct FileVisitor {
    docset_name: String
}

impl<'de> Visitor<'de> for FileVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string key and a string value")
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        build_docset_from_map_with_progress(&self.docset_name, map)
            .map_err(|err| Error::custom(format!("{err}")))?;
        Ok(())
    }
}

fn build_docset_from_db_json(
    docset_name: &String,
) -> ResultS {
    let docset_path = get_docset_path(&docset_name)?;
    let db_json_path = docset_path
        .join("db")
        .with_extension("json");

    let file = File::open(&db_json_path)
        .map_err(|err| format!("Could not open `{}`: {err}", db_json_path.display()))?;

    let reader = BufReader::new(file);

    let mut db_json_deserializer = serde_json::Deserializer::from_reader(reader);

    let file_visitor = FileVisitor { docset_name: docset_name.to_owned() };
    db_json_deserializer.deserialize_map(file_visitor)
        .map_err(|err| format!("Could not deserialize `{}`: {err}", db_json_path.display()))?;

    remove_file(&db_json_path)
        .map_err(|err| format!("Could not remove `{}` after building {docset_name}: {err}", db_json_path.display()))?;

    Ok(())
}

pub fn download<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    let mut flag_help;
    let mut flag_force;

    let mut flags = flags![
        flag_help: BoolFlag,  ["--help"],
        flag_force: BoolFlag, ["--force", "-f"]
    ];

    let args = parse_flags(&mut args, &mut flags)?;
    if flag_help || args.is_empty() { return show_download_help(); }

    if !is_docs_json_exists()? {
        return Err("`docs.json` does not exist. Please run `fetch` first".to_string());
    }

    let docs = deserialize_docs_json()?;

    let mut args_iter = args.iter();
    let mut successful_downloads = 0;

    while let Some(docset) = args_iter.next() {
        if !flag_force && is_docset_downloaded(docset)? {
            println!("\
{YELLOW}WARNING{RESET}: Docset `{docset}` is already downloaded. \
If you still want to update it, re-run this command with `--force`"
            );
            continue;
        } else {
            if is_docset_in_docs_or_print_warning(docset, &docs) {
                println!("Downloading `{docset}`...");
                download_db_and_index_json_with_progress(docset, &docs)?;

                println!("Extracting to `{}`...", get_docset_path(docset)?.display());
                build_docset_from_db_json(docset)?;

                successful_downloads += 1;
            }
        }
    }

    if successful_downloads > 1 {
        println!("{BOLD}{successful_downloads} items were successfully installed{RESET}.");
    } else if successful_downloads == 1 {
        println!("{BOLD}Install has successfully finished{RESET}.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let html_text = r#"
        <p id="what" class="hello">
            What
        </p>
        <h1 class="heading" href="hello world">
            <h2 id="heading-in-a-heading">
                What
            </h2>
        </h1 something="lol">
        "#;

        let should_be = r#"
        <p  >
            What
        </p>
        <h1  href="hello world">
            <h2 >
                What
            </h2>
        </h1 something="lol">
        "#;

        let result = sanitize_html_line(html_text.to_owned());

        assert_eq!(result, should_be);
    }
}
