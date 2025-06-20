use std::process::ExitCode;

use toiletcli::common::overwrite_should_use_colors;
use toiletcli::flags;
use toiletcli::flags::{parse_flags_until_subcommand, FlagType};

mod common;

use common::get_flag_error;
use common::ResultS;
use common::{BOLD, BUILD_TYPE, GREEN, HEAD, PROGRAM_NAME, RED, RESET, UNDERLINE, VERSION};

mod download;
mod fetch;
mod list;
mod open;
mod remove;
mod render;
mod search;

use download::download;
use fetch::fetch;
use list::list;
use open::open;
use remove::remove;
use render::render;
use search::search;

#[cfg(debug_assertions)]
use common::FLAG_INTEGRATION_TEST;

#[cfg(not(unix))]
#[cfg(not(windows))]
const OS: ! = panic!("Temple OS is not supported.");

fn show_short_version() -> ResultS
{
  let commit = HEAD.unwrap_or("unknown commit");
  println!("{VERSION}-{BUILD_TYPE} {commit}");
  Ok(())
}

fn show_version() -> ResultS
{
  let commit = HEAD.unwrap_or("unknown commit");
  println!(
           "\
dedoc {VERSION}, {BUILD_TYPE} build on {commit}
(c) toiletbril <{UNDERLINE}https://github.com/toiletbril{RESET}>

License GPLv3: GNU GPL version 3.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Report bugs and ask for features at <{UNDERLINE}https://github.com/toiletbril/dedoc{RESET}>.
Have a great day!"
  );
  Ok(())
}

fn show_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME}{RESET} [-OPTIONS] <subcommand> [-OPTIONS] [args]
    Search and display DevDocs pages from terminal.

{GREEN}SUBCOMMANDS{RESET}
    ft, fetch                       Fetch a list of available docsets.
    ls, list                        Display docsets from the fetched list.
    dl, download                    Download or update a docset from the list.
    rm, remove                      Delete local docsets.
    ss, search                      List or display docset pages that match a
                                    query.
    op, open                        Display docset pages.
    rr, render                      Render entire docsets to text.

  Each subcommand has its own `--help` option. Upon the first usage, please run
  `dedoc fetch`.

{GREEN}OPTIONS{RESET}
    -c, --force-colors              Forcefully enable colors.
        --color <on/off/auto>       Control output colors.
    -V, --short-version             Display short version.
    -v, --version                   Display version and license.
        --help                      Display help message."
  );
  Ok(())
}

fn entry<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  #[cfg(unix)]
  unsafe {
    // This feature is available only to Soyjak Gem 💎 users.
    // See issue #62569 <https://github.com/rust-lang/rust/issues/62569> for
    // more information. [E0621]
    libc::signal(libc::SIGPIPE, libc::SIG_IGN);
  }

  let mut flag_short_version;
  let mut flag_version;
  let mut flag_color;
  let mut flag_color_force;
  let mut flag_help;

  let mut flags = flags![
    flag_short_version: BoolFlag, ["-V", "--short-version"],
    flag_version: BoolFlag,       ["-v", "--version"],
    flag_color_force: BoolFlag,   ["-c", "--force-colors"],
    flag_color: StringFlag,       ["--color"],
    flag_help: BoolFlag,          ["--help"]
  ];

  // Every day I like Rust less and less.
  #[cfg(debug_assertions)]
  #[allow(static_mut_refs)]
  unsafe {
    flags.push((FlagType::BoolFlag(&mut FLAG_INTEGRATION_TEST), vec!["--integration-test"]));
  }

  let subcommand =
    parse_flags_until_subcommand(&mut args, &mut flags).map_err(|err| get_flag_error(&err))?
                                                       .to_lowercase();

  if flag_color_force {
    unsafe { overwrite_should_use_colors(true) }
  } else if !flag_color.is_empty() {
    match flag_color.as_str() {
      "auto" => {}
      "y" | "yes" | "on" => unsafe { overwrite_should_use_colors(true) },
      "n" | "no" | "off" => unsafe { overwrite_should_use_colors(false) },
      other => {
        return Err(format!("Argument `{other}` for `--color <on/off/auto>` is invalid."));
      }
    }
  }
  if flag_version {
    return show_version();
  }
  if flag_short_version {
    return show_short_version();
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
    "rr" | "render" => render(args),
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
