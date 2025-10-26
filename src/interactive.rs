use std::fs::File;
use std::io::Write;

use toiletcli::flags::{
  FlagType,
  parse_flags,
};

use crate::flags;

use crate::common::{
  BOLD,
  PROGRAM_NAME,
  RESET,
  ResultS,
};

use crate::print_warning;

const SCRIPT: &str = include_str!("../dedoc-interactive");

// Not actually interactive. Embed a script that uses fzf and less, ask to upack
// it along the main binary.
// Provide .bat wrapper for Windows.

pub(crate) fn interactive<Args>(mut args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  let mut flag_accept;

  let mut flags = flags![
    flag_accept: BoolFlag, ["--accept"]
  ];

  let _ = parse_flags(&mut args, &mut flags);

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
    f.write_all(SCRIPT.as_bytes()).expect("file is created by self");

    #[cfg(unix)]
    {
      // Set executable permissions
      use std::os::unix::fs::PermissionsExt;
      let mut perms = f.metadata().expect("why").permissions();
      perms.set_mode(0o755);
      f.set_permissions(perms).expect("the file is created by self");
    }
    #[cfg(windows)]
    {
      // Create .bat wrapper that calls sh.exe

      let wrapper_path = exe_dir.join(format!("{PROGRAM_NAME}-interactive.bat"));
      let mut wrapper = File::create(&wrapper_path).map_err(|err| {
                                                     format!("Could not create `{}`: {err}",
                                                             interactive_program_name.display())
                                                   })?;

      let wrapper_content =
        format!("@echo off\nsh.exe \"{}\"\n", interactive_program_name.display());
      wrapper.write_all(wrapper_content.as_bytes()).expect("file is created by self");
    }

    f.sync_all().expect("why");

    println!("{BOLD}The script was successfully unpacked to `{}` with execute \
              permissions.{RESET}",
             interactive_program_name.display());

    return Ok(());
  }

  // Otherwise just show the script contents.

  println!("```\n{}```\n", SCRIPT);

  print_warning!("The binary isn't actually interactive, but it provides an \
                  example script that allows it to become one.");
  print_warning!("Above are the contents of the script that is about to be \
                  unpacked in the same directory as the {PROGRAM_NAME} binary \
                  as `{PROGRAM_NAME}-interactive` (`{}`)",
                 &interactive_program_name.display());
  print_warning!("Re-run this command with `--accept` flag to unpack the script.");

  Ok(())
}
