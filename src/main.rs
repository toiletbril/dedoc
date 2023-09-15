use std::process::ExitCode;

extern crate toiletcli;

use toiletcli::common::overwrite_should_use_colors;
use toiletcli::flags::*;
use toiletcli::flags;

mod docs;
mod common;

use common::ResultS;
use common::{BOLD, UNDERLINE, GREEN, GRAY, PROGRAM_NAME, RED, RESET, VERSION};

mod open;
mod download;
mod remove;
mod search;
mod list;
mod fetch;

use open::open;
use remove::remove;
use download::download;
use search::search;
use list::list;
use fetch::fetch;

fn show_version() -> ResultS {
    let message = format!(
        "\
dedoc {VERSION}
(c) toiletbril <{UNDERLINE}https://github.com/toiletbril{RESET}>

Licensed under GPLv3.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law."
    );
    println!("{}", message);
    Ok(())
}

fn show_help() -> ResultS {
    let help = format!(
        "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME}{RESET} <subcommand> [args]
    Search DevDocs pages from terminal.

{GREEN}SUBCOMMANDS{RESET}
    {BOLD}fetch{RESET}{GRAY}, ft{RESET}                       Fetch available docsets.
    {BOLD}list{RESET}{GRAY}, ls{RESET}                        Show available docsets.
    {BOLD}download{RESET}{GRAY}, dl{RESET}                    Download docsets.
    {BOLD}remove{RESET}{GRAY}, rm{RESET}                      Delete docsets.
    {BOLD}search{RESET}{GRAY}, ss{RESET}                      List pages that match your query.
    {BOLD}open{RESET}{GRAY}, op{RESET}                        Display specified pages.

{GREEN}OPTIONS{RESET}
    -c, --color <on/off/auto>       Use color when displaying output.
    -v, --version                   Display version.
        --help                      Display help message. Can be used with subcommands.

The design is not final, and may be subject to change."
);
    println!("{}", help);
    Ok(())
}

fn entry<Args>(mut args: Args) -> ResultS
where
    Args: Iterator<Item = String>,
{
    debug_println!("{RED}Using debug build of {PROGRAM_NAME} v{VERSION}.{RESET}");

    let mut flag_version;
    let mut flag_help;
    let mut flag_color;

    let mut flags = flags![
        flag_help: BoolFlag,    ["--help"],
        flag_version: BoolFlag, ["--version", "-v"],
        flag_color: StringFlag, ["--color", "-c"]
    ];

    let subcommand = parse_flags_until_subcommand(&mut args, &mut flags)?
        .to_lowercase();

    if !flag_color.is_empty() {
        match flag_color.as_str() {
            "y" | "yes" | "on"  | "always" => unsafe { overwrite_should_use_colors(true) }
            "n" | "no"  | "off" | "never"  => unsafe { overwrite_should_use_colors(false) }
            "auto" | "tty" => {}
            other => {
                return Err(format!("Argument `{other}` for `--color <on/off/auto>` is invalid."));
            }
        }
    }
    if flag_version { return show_version(); }
    if flag_help || subcommand.is_empty() { return show_help(); }

    match subcommand.as_str() {
        "ft" | "fetch"    => fetch(args),
        "ls" | "list"     => list(args),
        "dl" | "download" => download(args),
        "rm" | "remove"   => remove(args),
        "ss" | "search"   => search(args),
        "op" | "open"     => open(args),
        other => {
            Err(format!("Unknown subcommand `{other}`."))
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let _ = &args.next().expect("Program path is provided");

    match entry(&mut args) {
        Err(err) => {
            eprintln!("{RED}ERROR{RESET}: {err}");
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}
