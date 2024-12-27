use lister::ManLister;
use manual::Manual;

use crate::ui::debug::log_to_file;

mod lister;
mod manual;

pub fn test() -> anyhow::Result<()> {
    let commands = ManLister::user_commands(None)?;
    for command in commands {
        println!("{command:?}");
    }

    Ok(())
}

pub(crate) fn list_user_commands() -> anyhow::Result<Vec<String>> {
    ManLister::user_commands(None)
}

pub(crate) fn get_manual(command: &str, width: &str) -> anyhow::Result<String> {
    Manual::fetch(command, width)
}
