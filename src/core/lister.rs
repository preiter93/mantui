use anyhow::{anyhow, Result};
use std::{collections::HashSet, process::Command};

pub(super) struct ManLister;

impl ManLister {
    pub(super) fn section(section: String) -> Result<Vec<String>> {
        let output = Command::new("man")
            .arg("-k")
            .arg("-S")
            .arg(section)
            .arg(".")
            .output()
            .expect("Failed to execute `man -k .`");

        if !output.status.success() {
            return Err(anyhow!(
                "command exited with a non-zero status: {}",
                output.status
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let commands: HashSet<String> = stdout
            .lines()
            .filter_map(|line| line.split(" - ").next())
            .filter_map(|line| line.split(", ").next())
            .filter(|cmd| {
                !cmd.starts_with(|c: char| {
                    matches!(
                        c,
                        ' ' | '!' | '$' | '*' | '%' | ':' | '<' | '-' | '/' | '.' | '@' | '['
                    )
                }) && !cmd.starts_with("Yet another")
                    && !cmd.starts_with("Other_name")
            })
            .map(str::trim)
            .map(String::from)
            .collect();

        let mut commands: Vec<_> = commands.into_iter().collect();
        commands.sort();

        Ok(commands)
    }
}
