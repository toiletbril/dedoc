use std::process::Command;

use crate::common::ResultS;

// man who needs to implement a pager from scratch when you can just inject a
// CVE into your program?
const SCRIPT: &'static str = include_str!("../dedoc-interactive");

#[cfg(unix)]
pub(crate) fn interactive<Args>(mut _args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  use std::os::unix::process::CommandExt;

  Command::new("sh").arg("-c")
                    .arg(SCRIPT)
                    .exec()
                    .map_err(|err| format!("Could not spawn shell subprocess: {err}"))?;

  unreachable!()
}

#[cfg(windows)]
pub(crate) fn interactive<Args>(mut _args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  use std::os::windows::process::CommandExt;

  let mut c =
    Command::new("sh").arg("-c")
                      .arg(SCRIPT)
                      .creation_flags(0x0)
                      .spawn()
                      .map_err(|err| format!("Could not spawn POSIX shell subprocess: {err}"))?;

  c.wait().map_err(|err| format!("Could not wait for subprocess: {err}"))?;

  Ok(())
}

// only windows and unix are supported.
