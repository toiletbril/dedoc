#![allow(unused)]
#![cfg(debug_assertions)]

use std::io::Write;
use std::fs::remove_dir_all;
use std::vec::IntoIter;

use crate::common::{get_program_directory, ResultS};

// This is needed to substitute stdout with a different buffer for testting.
pub static mut DEBUG_OUTPUT: Option<Box<Output>> = None;

// To capture stdout, replace DEBUG_OUTPUT with MockOutput using set_output_to_mock_output
pub struct MockOutput(Vec<u8>);

impl MockOutput {
    pub fn new() -> Self {
        MockOutput(vec![])
    }
    pub fn as_string(&self) -> String {
        String::from_utf8(self.0.clone()).unwrap()
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Write for MockOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub enum Output {
    Stdout(std::io::Stdout),
    MockOutput(MockOutput)
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout(stdout) => stdout.write(buf),
            Self::MockOutput(output) => output.write(buf)
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.flush(),
            Self::MockOutput(output) => output.flush()
        }
    }
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.write_fmt(fmt),
            Self::MockOutput(output) => output.write_fmt(fmt),
        }
    }
}

pub unsafe fn set_output_to_stdout() {
    DEBUG_OUTPUT = Some(Box::new(Output::Stdout(std::io::stdout())));
}

pub unsafe fn set_output_to_mock_output() {
    DEBUG_OUTPUT = Some(Box::new(Output::MockOutput(MockOutput::new())));
}

pub fn get_mock_output() -> String {
    unsafe {
        let output = DEBUG_OUTPUT.as_ref().unwrap();
        if let Output::MockOutput(ref output) = **output {
            return output.as_string();
        }
    }
    panic!("Output is not MockOutput")
}

pub fn clear_mock_output() {
    unsafe {
        let output = DEBUG_OUTPUT.as_mut().unwrap();
        if let Output::MockOutput(ref mut output) = **output {
            return output.clear();
        }
    }
    panic!("Output is not MockOutput")
}

pub fn create_args<'a>(args: &'a str) -> IntoIter<String> {
    args.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .into_iter()
}

pub fn remove_program_dir() -> ResultS {
    let program_directory = get_program_directory()?;
    remove_dir_all(&program_directory)
        .map_err(|err| format!("Could not remove `{}`: {err}", program_directory.display()))
}

#[inline]
pub fn _dedoc_write(out: &mut dyn ::std::io::Write, data: &str) {
    let _ = write!(out, "{}", data);
}

#[inline]
pub fn _dedoc_writeln(out: &mut dyn ::std::io::Write, data: &str) {
    let _ = writeln!(out, "{}", data);
}

#[macro_export]
macro_rules! dedoc_println_impl {
    () => {
        unsafe {
            if let Some(ref mut out) = $crate::debug::DEBUG_OUTPUT {
                $crate::debug::_dedoc_write(out, "\n");
            }
        }
    };
    ($($e:expr),+) => {
        unsafe {
            if let Some(ref mut out) = $crate::debug::DEBUG_OUTPUT {
                $crate::debug::_dedoc_writeln(out, &format!($($e),+));
            }
        }
    };
}

#[macro_export]
macro_rules! dedoc_print_impl {
    ($($e:expr),+) => {
        unsafe {
            if let Some(ref mut out) = $crate::debug::DEBUG_OUTPUT {
                let _ = $crate::debug::_dedoc_write(out, &format!($($e),+));
            }
        }
    }
}

