use std::fs::File;
use std::io::Write;

use toiletcli::flags;
use toiletcli::flags::*;

use crate::common::ResultS;

use crate::common::{
  BOLD,
  GREEN,
  PROGRAM_NAME,
  RESET,
};

use crate::print_warning;

#[cfg(unix)]
const SCRIPT_NAME: &str = "dedoc-interactive";
#[cfg(windows)]
const SCRIPT_NAME: &str = "dedoc-interactive.ps1";

#[cfg(unix)]
const SCRIPT_CONTENTS: &str = include_str!("../dedoc-interactive");
#[cfg(windows)]
const SCRIPT_CONTENTS: &str = include_str!("../dedoc-interactive.ps1");

// Embed a script that uses fzf and less, ask to upack it along the main
// binary. A .bat wrapper is provided for Windows.

fn show_list_help() -> ResultS
{
  println!(
           "\
{GREEN}USAGE{RESET}
    {BOLD}{PROGRAM_NAME} install{RESET} [--accept]
    Install helper scripts.

{GREEN}OPTIONS{RESET}
        --accept                    Agree and actually unpack the scripts.
        --help                      Display help message."
  );
  Ok(())
}

pub(crate) fn install<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_accept;
  let mut flag_help;

  let mut flags = flags![
    flag_accept: BoolFlag, ["--accept"],
    flag_help: BoolFlag,   ["--help"]
  ];

  let _ = parse_flags(&mut args, &mut flags);

  if flag_help {
    return show_list_help();
  }

  let exe_path =
    std::env::current_exe().map_err(|err| format!("Could not get current path: {err}"))?;
  let exe_dir = exe_path.parent().expect("how");
  let script_path = exe_dir.join(SCRIPT_NAME);

  // Ready to unpack?
  if flag_accept {
    let mut f = File::create(&script_path).map_err(|err| {
                                            format!("Could not create `{}`: {err}",
                                                    script_path.display())
                                          })?;
    f.write_all(SCRIPT_CONTENTS.as_bytes()).expect("file is created by self");

    #[cfg(unix)]
    {
      // Set executable permissions for the script.

      use std::os::unix::fs::PermissionsExt;
      let mut perms = f.metadata().expect("why").permissions();
      perms.set_mode(0o755);
      f.set_permissions(perms).expect("the file is created by self");
    }

    f.sync_all().expect("why");

    println!("{BOLD}The script was successfully unpacked to `{}` with execute \
              permissions.{RESET}",
             script_path.display());

    return Ok(());
  }

  // Otherwise just show the script contents.

  println!("```\n{}```\n", SCRIPT_CONTENTS);

  print_warning!("The binary provides an example script that allows it to become \
                  interactive.");
  print_warning!("Above are the contents of the script that is about to be \
                  unpacked in the same directory as the {PROGRAM_NAME} binary \
                  as `{PROGRAM_NAME}-interactive` (`{}`)",
                 &script_path.display());

  print_warning!("You'll need `skim`/`fzf` as fuzzy searcher and `moar`/`less` \
                  as a pager.");

  print_warning!("Re-run this command with `--accept` flag to unpack the script.");

  Ok(())
}
