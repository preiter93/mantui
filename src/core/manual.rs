use anyhow::{Result, anyhow};
use ratatui::{
    style::{Modifier, Style},
    text::Span,
};
use std::{
    io::Read,
    process::{Command, Stdio},
};

use crate::ui::debug::log_to_file;

pub(crate) struct Manual;

impl Manual {
    pub(super) fn fetch(command: &str, width: &str) -> Result<String> {
        let process = Command::new("man")
            .arg(command)
            .env("MANWIDTH", width)
            .env("LC_ALL", "C")
            .stdout(Stdio::piped())
            .spawn()?;

        let mut output = String::new();
        process.stdout.unwrap().read_to_string(&mut output)?;

        Ok(man_to_ansi(&output))
    }
}

const BOLD: &str = "\x1B[1m";
const RESET: &str = "\x1B[0m";
const RED: &str = "\x1B[31m";

fn man_to_ansi(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().rev().peekable();

    while let Some(curr) = chars.next() {
        match (curr, chars.peek()) {
            (ch, Some('\u{8}')) => {}
            ('\u{8}', Some(ch)) => {
                result.insert_str(0, &format!("{BOLD}{ch}{RESET}"));
                chars.next();
            }
            (ch, _ | None) => result.insert(0, ch),
            _ => unreachable!(),
        }
    }

    remove_redundant_ansi(&mut result);

    result
}

fn remove_redundant_ansi(result: &mut String) {
    *result = result.replace("{RESET}{BOLD}", "");
}

mod test {
    use super::man_to_ansi;

    #[test]
    fn test_man_to_ansi() {
        let man = "COMMAND N\u{8}NA\u{8}AM\u{8}ME\u{8}E";
        let ansi = man_to_ansi(man);

        assert_eq!(ansi, String::from("COMMAND \x1B[1mNAME\x1B[0m"));
    }
}
