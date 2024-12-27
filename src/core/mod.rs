use lister::ManLister;
use manual::Manual;

mod lister;
mod manual;

pub(crate) fn list_user_commands() -> anyhow::Result<Vec<String>> {
    ManLister::user_commands(1)
}

pub(crate) fn get_manual(command: &str, width: &str) -> anyhow::Result<String> {
    let manual = Manual::fetch(command, width)?;
    if manual.is_empty() {
        Manual::fetch(&command.to_lowercase(), width)
    } else {
        Ok(manual)
    }
}
