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

const SCRIPT_INTERACTIVE: &str = include_str!("../dedoc-interactive");

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
  let interactive_program_name = exe_dir.join(format!("{PROGRAM_NAME}-interactive"));

  // Ready to unpack?
  if flag_accept {
    let mut f = File::create(&interactive_program_name).map_err(|err| {
                                                         format!("Could not create `{}`: {err}",
                                                                 interactive_program_name.display())
                                                       })?;
    f.write_all(SCRIPT_INTERACTIVE.as_bytes()).expect("file is created by self");

    #[cfg(unix)]
    {
      // Set executable permissions for the script.

      use std::os::unix::fs::PermissionsExt;
      let mut perms = f.metadata().expect("why").permissions();
      perms.set_mode(0o755);
      f.set_permissions(perms).expect("the file is created by self");
    }

    #[cfg(windows)]
    {
      // Windows cannot launch shell files by itself. Create .bat wrapper that
      // calls sh.exe with out script.

      let wrapper_path = exe_dir.join(format!("{PROGRAM_NAME}-interactive.bat"));
      let mut wrapper = File::create(&wrapper_path).map_err(|err| {
                                                     format!("Could not create `{}`: {err}",
                                                             interactive_program_name.display())
                                                   })?;

      let wrapper_content =
        format!("@echo off\nsh.exe \"{}\"\n", interactive_program_name.display());
      wrapper.write_all(wrapper_content.as_bytes()).expect("file is created by self");

      wrapper.sync_all.expect("why");

      println!("Installed wrapper script at `{}`.", wrapper_path.display());
    }

    f.sync_all().expect("why");

    println!("{BOLD}The script was successfully unpacked to `{}` with execute \
              permissions.{RESET}",
             interactive_program_name.display());

    return Ok(());
  }

  // Otherwise just show the script contents.

  println!("```\n{}```\n", SCRIPT_INTERACTIVE);

  print_warning!("The binary provides an example script that allows it to become \
                  interactive.");
  print_warning!("Above are the contents of the script that is about to be \
                  unpacked in the same directory as the {PROGRAM_NAME} binary \
                  as `{PROGRAM_NAME}-interactive` (`{}`)",
                 &interactive_program_name.display());

  #[cfg(windows)]
  print_warning!("You'll need `busybox`/`cosmopolitan`/`mingw` POSIX sh binary to \
                  run the script itself, `skim`/`fzf` as fuzzy searcher and \
                  moar/less as a pager.");
  #[cfg(unix)]
  print_warning!("You'll need `skim`/`fzf` as fuzzy searcher and `moar`/`less` \
                  as a pager.");

  print_warning!("Re-run this command with `--accept` flag to unpack the script.");

  Ok(())
}
