use anyhow::anyhow;
use std::process::Command;

use anyhow::Result;

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

        let commands = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(String::from)
            .collect();

        Ok(commands)
    }
}
