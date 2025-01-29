use anyhow::{anyhow, Result};
use std::process::Command;

pub(crate) struct Reader;

impl Reader {
    pub(super) fn read(command: &str, width: &str) -> Result<String> {
        let output = if cfg!(target_os = "macos") {
            command_macos(command, width)
                .or_else(|_| command_macos(command.to_lowercase(), width))?
        } else {
            command_linux(command, width)
                .or_else(|_| command_linux(&command.to_lowercase(), width))
                .or_else(|_| command_macos(command, width))
                .or_else(|_| command_macos(command.to_lowercase(), width))?
        };

        let ansi = man_to_ansi(&output);

        Ok(ansi)
    }
}

fn command_macos<S: AsRef<str>>(command: S, width: &str) -> Result<String> {
    let output = Command::new("man")
        .arg(strip_section(command.as_ref()))
        .env("MANWIDTH", width)
        .env("LC_ALL", "C")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("command failed: {}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn command_linux(command: &str, width: &str) -> Result<String> {
    let output = Command::new("man")
        .arg("-t")
        .arg("-Tutf8")
        .arg(strip_section(command))
        .env("MANWIDTH", width)
        .env("LC_ALL", "C")
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("command failed: {}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn strip_section(command: &str) -> String {
    if command.ends_with(')') {
        if let Some(pos) = command.rfind('(') {
            return command[..pos].trim().to_string();
        }
    }
    command.trim().to_string()
}

const ANSI_RESET: &str = "\x1B[0m";
#[allow(unused)]
const ANSI_BOLD: &str = "\x1B[1m";
#[allow(unused)]
const ANSI_ITALIC: &str = "\x1B[3m";
#[allow(unused)]
const ANSI_UNDERLINE: &str = "\x1B[4m";

const ANSI_ORANGE: &str = "\x1B[38;2;248;154;99m";

const ANSI_PURPLE: &str = "\x1b[38;2;152;120;209m";

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
                        result.push_str(&formatted_char(*ch, ANSI_PURPLE));
                    }
                    Format::Underline => {
                        result.push_str(&formatted_char(*ch, ANSI_ORANGE));
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
    *result = result.replace(&format!("{ANSI_RESET}{ANSI_PURPLE}"), "");
    *result = result.replace(&format!("{ANSI_RESET}{ANSI_ORANGE}"), "");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_man_to_ansi() {
        let man = "COMMAND N\u{8}NA\u{8}AM\u{8}ME\u{8}E";
        let ansi = man_to_ansi(man);

        assert_eq!(
            ansi,
            String::from(format!("COMMAND {ANSI_PURPLE}NAME{ANSI_RESET}"))
        );
    }

    #[test]
    fn test_man_to_ansi_underline() {
        let man = "_\u{8}N_\u{8}A_\u{8}M_\u{8}E";
        let ansi = man_to_ansi(man);

        assert_eq!(ansi, String::from(format!("{ANSI_ORANGE}NAME{ANSI_RESET}")));
    }
}
