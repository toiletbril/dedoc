use std::fs::{create_dir_all, remove_dir_all, remove_file, File};
use std::io::{BufReader, BufWriter, Read, Write};

use attohttpc::get;

use flate2::bufread::GzDecoder;
use tar::Archive;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{Docs, ResultS};
use crate::common::{
    deserialize_docs_json, get_docset_path, get_program_directory, is_docs_json_exists,
    is_docset_downloaded, is_docset_in_docs_or_print_warning,
};

use crate::common::{
    BOLD, DEFAULT_DOWNLOADS_LINK, DEFAULT_USER_AGENT, GREEN, PROGRAM_NAME, RESET, VERSION, YELLOW,
};

fn show_download_help() -> ResultS {
    println!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} download{RESET} [-f] <docset1> [docset2, ..]
    Download a docset. Available docsets can be displayed using `list`.

{GREEN}OPTIONS{RESET}
    -f, --force                     Overwrite downloaded docsets.
        --help                      Display help message."
    );
    Ok(())
}

fn download_docset_tar_gz_with_progress(
    docset_name: &String,
    docs: &Vec<Docs>,
) -> Result<(), String> {
    let user_agent = format!("{DEFAULT_USER_AGENT}/{VERSION}");

    for entry in docs.iter() {
        if docset_name == &entry.slug {
            let docsets_path = get_program_directory()?.join("docsets");
            let specific_docset_path = docsets_path.join(&docset_name);

            if !specific_docset_path.exists() {
                create_dir_all(&specific_docset_path)
                    .map_err(|err| format!("Cannot create `{docset_name}` directory: {err}"))?;
            }

            let tar_gz_path = specific_docset_path
                .join(docset_name)
                .with_extension("tar.gz");

            let file = File::create(&tar_gz_path)
                .map_err(|err| format!("Could not create `{}`: {err}", tar_gz_path.display()))?;

            let download_link = format!("{DEFAULT_DOWNLOADS_LINK}/{docset_name}.tar.gz");

            let response = get(&download_link)
                .header_append("user-agent", &user_agent)
                .send()
                .map_err(|err| format!("Could not GET {download_link}: {err}"))?;

            let content_length = response
                .headers()
                .get("content-length")
                .map_or("0", |header| header.to_str().unwrap_or("0"))
                .parse::<usize>()
                .unwrap_or(0);

            let mut file_writer = BufWriter::new(file);
            let mut response_reader = BufReader::new(response);

            let mut buffer = [0; 1024 * 8];
            let mut file_size = 0;

            while let Ok(size) = response_reader.read(&mut buffer) {
                if size == 0 {
                    break;
                }

                file_writer
                    .write(&buffer[..size])
                    .map_err(|err| format!("Could not download file: {err}"))?;

                file_size += size;

                print!("\rReceived {file_size} of {content_length} bytes...");
            }
            println!();

            file_writer
                .flush()
                .map_err(|err| format!("Could not flush buffer to file: {err}"))?;

            if file_size != content_length {
                let message = format!(
                    "File size ({file_size}) is different than required size ({content_length}). \
                     Please re-run this command :("
                );

                remove_dir_all(&specific_docset_path)
                    .map_err(|err| format!("Could not remove bad docset ({}): {err}", specific_docset_path.display()))?;

                return Err(message);
            }
        }
    }

    Ok(())
}

fn extract_docset_tar_gz(docset_name: &String) -> Result<(), String> {
    let docset_path = get_docset_path(docset_name)?;

    if !docset_path.exists() {
        create_dir_all(&docset_path)
            .map_err(|err| format!("Cannot create `{docset_name}` directory: {err}"))?;
    }

    let tar_gz_path = docset_path.join(docset_name).with_extension("tar.gz");

    let tar_gz_file = File::open(&tar_gz_path)
        .map_err(|err| format!("Could not open `{}`: {err}", tar_gz_path.display()))?;

    let reader = BufReader::new(tar_gz_file);
    let tar = GzDecoder::new(reader);
    let mut archive = Archive::new(tar);

    #[cfg(target_family = "unix")]
    {
        archive
            .unpack(docset_path)
            .map_err(|err| format!("Could not extract `{}`: {err}", tar_gz_path.display()))?;
    }

    // Sometimes .tar archives have files with disallowed characters in their name.
    // Unpack them manually while replacing invalid characters.
    #[cfg(target_family = "windows")]
    {
        const FORBIDDEN_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

        let mut archive_files = archive.entries()
            .map_err(|err| format!("Could not read archive `{}`: {err}", tar_gz_path.display()))?;

        while let Some(file) = archive_files.next() {
            let mut file = file
                .map_err(|err| format!("Could not read archive `{}`: {err}", tar_gz_path.display()))?;

            let path_bytes = &file.header().path_bytes();
            let path_unsanitized_str = String::from_utf8_lossy(&path_bytes);

            let path = path_unsanitized_str
                .chars()
                .map(|c| if FORBIDDEN_CHARS.contains(&c) { '_' } else { c })
                .collect::<String>();

            let path_sanitized = path
                .strip_suffix(&['.', ' '])
                .unwrap_or(path.as_str())
                .trim();

            let unpack_path = docset_path.join(path_sanitized);

            file.unpack(&unpack_path)
                .map_err(|err| format!("Could not extract file from `{}` to {unpack_path:?}: {err}", tar_gz_path.display()))?;
        }
    }

    remove_file(&tar_gz_path)
        .map_err(|err| format!("Could not remove `{}`: {err}", tar_gz_path.display()))?;

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
{YELLOW}WARNING{RESET}: `{docset}` is already downloaded. If you still want to update it, re-run this command with `--force`"
            );
            continue;
        } else {
            if is_docset_in_docs_or_print_warning(docset, &docs) {
                println!("Downloading `{docset}`...");
                download_docset_tar_gz_with_progress(docset, &docs)?;

                println!("Extracting to `{}`...", get_docset_path(docset)?.display());
                extract_docset_tar_gz(docset)?;

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
