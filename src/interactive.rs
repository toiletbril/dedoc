use crate::common::ResultS;

const SCRIPT: &'static str = include_str!("../dedoc-interactive");

#[cfg(unix)]
pub(crate) fn interactive<Args>(mut _args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  use std::{
    os::unix::process::CommandExt,
    process::Command,
  };

  let _ = Command::new("sh").arg("-c").arg(SCRIPT).exec();

  unreachable!()
}

#[cfg(not(unix))]
pub(crate) fn interactive<Args>(mut _args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  Err(format!("Your OS is not supported for built-in interactive mode. However, \
               you may `dedoc-interactive` from the source repository manually."))
}
