use anyhow::Result;
use std::{
    io::Read,
    process::{Command, Stdio},
};

pub(crate) struct Manual;

impl Manual {
    pub(super) fn fetch(command: &str, width: &str) -> Result<String> {
        let process = Command::new("man")
            .arg(command)
            .env("MANWIDTH", width)
            .env("LC_ALL", "C")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut output = String::new();
        process.stdout.unwrap().read_to_string(&mut output)?;

        let ansi = man_to_ansi(&output);

        Ok(ansi)
    }
}

const ANSI_RESET: &str = "\x1B[0m";
const ANSI_BOLD: &str = "\x1B[1m";
#[allow(unused)]
const ANSI_ITALIC: &str = "\x1B[3m";
const ANSI_UNDERLINE: &str = "\x1B[4m";

const ANSI_RED: &str = "\x1B[31m";
#[allow(unused)]
const ANSI_GREEN: &str = "\x1B[32m";

enum Format {
    None,
    Bold,
    Underline,
}

// Bold: c\u{8}c
// UnderLine: _\u{8}c
fn man_to_ansi(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    let mut active_format = Format::None;

    while let Some(curr) = chars.next() {
        match (curr, chars.peek()) {
            (ch, Some('\u{8}')) => {
                if ch == '_' {
                    active_format = Format::Underline;
                } else {
                    active_format = Format::Bold;
                }
            }
            ('\u{8}', Some(ch)) => {
                match active_format {
                    Format::Bold => {
                        result.push_str(&formatted_char(*ch, ANSI_BOLD));
                    }
                    Format::Underline => {
                        result.push_str(&formatted_char(*ch, ANSI_RED));
                    }
                    Format::None => {}
                }
                chars.next();
            }
            (ch, _) => result.push(ch),
        }
    }

    remove_redundant_ansi(&mut result);

    result = result.replace('`', "'");

    result
}

fn formatted_char(ch: char, format: &str) -> String {
    format!("{format}{ch}{ANSI_RESET}")
}

fn remove_redundant_ansi(result: &mut String) {
    *result = result.replace(&format!("{ANSI_RESET}{ANSI_BOLD}"), "");
    *result = result.replace(&format!("{ANSI_RESET}{ANSI_UNDERLINE}"), "");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_man_to_ansi() {
        let man = "COMMAND N\u{8}NA\u{8}AM\u{8}ME\u{8}E";
        let ansi = man_to_ansi(man);

        assert_eq!(ansi, String::from("COMMAND \x1B[1mNAME\x1B[0m"));
    }

    #[test]
    fn test_man_to_ansi_underline() {
        let man = "_\u{8}N_\u{8}A_\u{8}M_\u{8}E";
        let ansi = man_to_ansi(man);

        assert_eq!(ansi, String::from("\x1B[4mNAME\x1B[0m"));
    }
}
