use descriptor::ManDescriptor;
use lister::ManLister;

use crate::ui::debug::log_to_file;

mod descriptor;
mod lister;

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

pub(crate) fn get_man_page(command: &str) -> anyhow::Result<String> {
    ManDescriptor::describe(command)
}
