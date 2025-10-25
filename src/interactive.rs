use crate::common::ResultS;

const SCRIPT: &'static str = include_str!("../dedoc-interactive");

pub(crate) fn interactive<Args>(mut _args: Args) -> ResultS
  where Args: Iterator<Item = String>
{
  // XXX show the script and then unpack it
  todo!()
}
