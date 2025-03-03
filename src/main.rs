use std::process::ExitCode;

use toiletcli::common::overwrite_should_use_colors;
use toiletcli::flags;
use toiletcli::flags::{parse_flags_until_subcommand, FlagType};

mod common;

use common::get_flag_error;
use common::ResultS;
use common::{BOLD, GREEN, PROGRAM_NAME, RED, RESET, UNDERLINE, VERSION};

mod download;
mod fetch;
mod list;
mod open;
mod remove;
mod search;

#[cfg(debug_assertions)]
mod test;

use download::download;
use fetch::fetch;
use list::list;
use open::open;
use remove::remove;
use search::search;

#[cfg(debug_assertions)]
use test::debug_test;

#[cfg(not(unix))]
#[cfg(not(windows))]
const OS: ! = panic!("Temple OS is not supported.");

fn show_version() -> ResultS
{
  #[cfg(debug_assertions)]
  let version = format!("{VERSION} debug build");
  #[cfg(not(debug_assertions))]
  let version = VERSION;
  println!(
           "\
dedoc {version}
(c) toiletbril <{UNDERLINE}https://github.com/toiletbril{RESET}>

License GPLv3: GNU GPL version 3.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law."
  );
  Ok(())
}

fn show_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME}{RESET} <subcommand> [args]
    Search DevDocs pages from terminal.

{GREEN}SUBCOMMANDS{RESET}
    ft, fetch                       Fetch available docsets.
    ls, list                        Show docsets available for download.
    dl, download                    Download and update docsets.
    rm, remove                      Delete local docsets.
    ss, search                      List pages that match your query.
    op, open                        Display specified pages.

{GREEN}OPTIONS{RESET}
    -c, --force-colors              Forcefully enable colors.
        --color <on/off/auto>       Control output colors.
    -v, --version                   Display version.
        --help                      Display help message."
  );
  Ok(())
}

fn entry<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  debug_println!("Using debug build of {PROGRAM_NAME} v{VERSION}. Run `test` \
                  to perform tests.");

  let mut flag_version;
  let mut flag_color;
  let mut flag_color_force;
  let mut flag_help;

  let mut flags = flags![
    flag_version: BoolFlag,     ["-v", "--version"],
    flag_color_force: BoolFlag, ["-c", "--force-colors"],
    flag_color: StringFlag,     ["--color"],
    flag_help: BoolFlag,        ["--help"]
  ];

  let subcommand =
    parse_flags_until_subcommand(&mut args, &mut flags).map_err(|err| {
                                                         get_flag_error(&err)
                                                       })?
                                                       .to_lowercase();

  if flag_color_force {
    unsafe { overwrite_should_use_colors(true) }
  } else if !flag_color.is_empty() {
    match flag_color.as_str() {
      "auto" => {}
      "y" | "yes" | "on" => unsafe { overwrite_should_use_colors(true) },
      "n" | "no" | "off" => unsafe { overwrite_should_use_colors(false) },
      other => {
        return Err(format!(
          "Argument `{other}` for `--color <on/off/auto>` is invalid."
        ));
      }
    }
  }
  if flag_version {
    return show_version();
  }
  if flag_help || subcommand.is_empty() {
    return show_help();
  }

  match subcommand.as_str() {
    "ft" | "fetch" => fetch(args),
    "ls" | "list" => list(args),
    "dl" | "download" => download(args),
    "rm" | "remove" => remove(args),
    "ss" | "search" => search(args),
    "op" | "open" => open(args),
    #[cfg(debug_assertions)]
    "test" => debug_test(args),
    other => Err(format!("Unknown subcommand `{other}`")),
  }
}

fn main() -> ExitCode
{
  let mut args = std::env::args();
  let _ = &args.next().expect("Program path is provided");

  match entry(&mut args) {
    Err(mut err) => {
      if !err.ends_with(['.', '?', ')']) {
        err += ". Try `--help` for more information.";
      }
      eprintln!("{RED}ERROR{RESET}: {err}");
      ExitCode::FAILURE
    }
    _ => ExitCode::SUCCESS,
  }
}
