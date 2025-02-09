use std::fs::{create_dir, create_dir_all, read_dir, File};
use std::io::{stdout, Write};
use std::path::Path;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::{
  deserialize_docs_json, get_docset_path, get_flag_error, get_local_docsets, is_docs_json_exists,
  is_docset_downloaded, make_sure_docset_is_in_docs, translate_docset_file_to_markdown,
  DOC_PAGE_EXTENSION,
};
use crate::common::{get_program_directory, validate_number_of_columns, ResultS, MAX_WIDTH};
use crate::common::{BOLD, GREEN, PROGRAM_NAME, RESET};
use crate::print_warning;

fn show_render_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} render{RESET} [-cd] <docset1> [docset2, ...]
    Render a whole docset to markdown.

{GREEN}OPTIONS{RESET}
    -c, --columns                   Change output width in columns. Default is
                                    144.
    -d, --output-dir                Specify output directory. Default is
                                    `~/.dedoc/rendered/<docset>`.
        --all                       Render all docsets. In case of `-d`, a
                                    subdirectory will be created for each
                                    docset anyway.
        --help                      Display help message."
  );
  Ok(())
}

fn render_docset(docset: &str, output_dir: &Path, page_width: usize) -> ResultS
{
  fn render_docset_recurse(docset: &str,
                           docset_path: &Path,
                           path: &Path,
                           output_dir: &Path,
                           page_width: usize,
                           counter: &mut usize)
                           -> ResultS
  {
    let docset_dir = read_dir(&path).map_err(|err| {
                                      format!("Could not read `{}` directory: {err}",
                                              docset_path.display())
                                    })?;

    for entry in docset_dir {
      let entry =
        entry.map_err(|err| {
               format!("Could not traverse `{}`: {}", docset_path.display(), err.to_string())
             })?;

      let md_dir = entry.path().strip_prefix(&docset_path).expect("no way :(").to_owned();

      if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
        let md_dir_path = &output_dir.join(&md_dir);
        if !md_dir_path.try_exists()
                       .map_err(|err| {
                         format!("Could not check if {} exists: {err}", docset_path.display())
                       })?
        {
          create_dir(&md_dir_path).map_err(|err| {
                                    format!("Could not create subdirectory `{}`: {err}",
                                            md_dir_path.display())
                                  })?;
        }
        render_docset_recurse(docset, docset_path, &entry.path(), output_dir, page_width, counter)?;
        continue;
      }

      if entry.path().extension().unwrap_or_default() != DOC_PAGE_EXTENSION {
        continue;
      }

      print!("\rRendered {} files from `{}` into `{}`...", counter, docset, output_dir.display());
      stdout().flush().map_err(|err| format!("Could not flush stdout: {err}"))?;

      let md_dir = md_dir.parent().expect("uhh");
      let mut md_file_path = md_dir.join(entry.file_name().to_string_lossy().to_string());
      md_file_path.set_extension("md");

      let file_path = output_dir.join(md_file_path);

      let mut file = File::create(&file_path).map_err(|err| {
                                               format!("Could not create `{}`: {}",
                                                       file_path.display(),
                                                       err.to_string())
                                             })?;

      file.write(&translate_docset_file_to_markdown(entry.path(), None, page_width, false, false)?.0.as_bytes())
        .map_err(|err| format!("Could not write to `{}`: {}", file_path.display(), err.to_string()))?;

      let _ = file.flush();
      *counter += 1;
    }

    Ok(())
  }

  let mut counter = 0;

  render_docset_recurse(docset,
                        &get_docset_path(docset)?,
                        &get_docset_path(docset)?,
                        output_dir,
                        page_width,
                        &mut counter)?;
  println!();

  Ok(())
}

pub(crate) fn render<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_all;
  let mut flag_columns;
  let mut flag_output_dir;
  let mut flag_help;

  let mut flags = flags![
    flag_columns: StringFlag,    ["-c", "--columns"],
    flag_output_dir: StringFlag, ["-d", "--output-dir"],
    flag_all: BoolFlag,          ["--all"],
    flag_help: BoolFlag,         ["--help"]
  ];

  let args = parse_flags(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?;
  if flag_help || (args.is_empty() && !flag_all) {
    return show_render_help();
  }

  if !is_docs_json_exists()? {
    return Err(format!(
      "The list of available documents has not yet been downloaded. \
       Please run `{PROGRAM_NAME} fetch` first."
    ));
  }

  let changed_directory = !flag_output_dir.is_empty();

  let main_output_dir = if flag_output_dir.is_empty() {
    get_program_directory()?.join("rendered")
  } else {
    flag_output_dir.into()
  };

  let page_width =
    if flag_columns.is_empty() { MAX_WIDTH } else { validate_number_of_columns(&flag_columns)? };

  if changed_directory &&
     main_output_dir.try_exists()
                    .map_err(|err| {
                      format!("Could not check if `{}` exists: {err}", main_output_dir.display())
                    })?
  {
    return Err(format!("`{}` already exists. Please remove it before running the \
                        command to avoid flooding unrelated directories.",
                       main_output_dir.display()));
  }

  let local_docsets = get_local_docsets()?;

  if flag_all {
    let mut directories = vec![];

    if !args.is_empty() {
      print_warning!("Arguments are ignored due to `--all` flag.");
    }
    if local_docsets.is_empty() {
      return Err("Nothing to do.".to_string());
    }

    for docset in &local_docsets {
      directories.push(main_output_dir.join(docset));
    }

    create_dir_all(&main_output_dir).map_err(|err| {
                                      format!("Could not create output directory `{}`: {err}",
                                              main_output_dir.display())
                                    })?;

    for (ref docset, ref sub_dir) in local_docsets.into_iter().zip(directories) {
      create_dir_all(sub_dir).map_err(|err| {
                               format!("Could not create subdirectory `{}`: {err}",
                                       sub_dir.display())
                             })?;
      render_docset(docset, sub_dir, page_width)?;
    }
  } else {
    for docset in args {
      if !is_docset_downloaded(&docset)? {
        make_sure_docset_is_in_docs(&docset, &deserialize_docs_json()?)?;
        return Err(format!("Docset `{docset}` is not downloaded. Try running \
                          `{PROGRAM_NAME} download {docset}`."));
      }
      let output_dir =
        if !changed_directory { main_output_dir.join(&docset) } else { main_output_dir.clone() };
      create_dir_all(&output_dir).map_err(|err| {
                                   format!("Could not create subdirectory `{}`: {err}",
                                           output_dir.display())
                                 })?;
      render_docset(&docset, &output_dir, page_width)?;
    }
  }

  println!("{BOLD}Render has successfully finished{RESET}.");

  Ok(())
}
