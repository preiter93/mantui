use anyhow::{Result, anyhow};
use std::{collections::HashSet, process::Command};

pub(super) struct ManLister;

impl ManLister {
    pub(super) fn user_commands(search: Option<&str>) -> Result<Vec<String>> {
        let mut command = Command::new("apropos");
        command.arg("-s").arg("1");

        if let Some(search) = search {
            command.arg(search);
        } else {
            command.arg(".");
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "command exited with a non-zero status: {}",
                output.status
            ));
        }

        let mut commands = HashSet::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut split = line.split("(1)").collect::<Vec<_>>();
            if split.len() >= 2 {
                commands.insert(split[0].to_string());
            }
        }

        let mut commands: Vec<_> = commands.into_iter().collect();
        commands.sort();

        Ok(commands)
    }
}
