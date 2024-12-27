use anyhow::{Result, anyhow};
use std::{
    io::Read,
    process::{Command, Stdio},
};

pub(crate) struct ManDescriptor;

impl ManDescriptor {
    pub(super) fn describe(command: &str) -> Result<String> {
        // Execute the `man` command
        let man_process = Command::new("man")
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()?;

        // Pipe the output to `col -bx`
        let col_process = Command::new("col")
            .arg("-bx")
            .stdin(Stdio::from(man_process.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()?;

        let mut output = String::new();
        col_process.stdout.unwrap().read_to_string(&mut output)?;

        Ok(output)
    }
}
