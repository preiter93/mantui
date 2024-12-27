use lister::ManLister;
use manual::Manual;

mod lister;
mod manual;

pub(crate) fn load_section(section: String) -> anyhow::Result<Vec<String>> {
    ManLister::section(section)
}

pub(crate) fn get_manual(command: &str, width: &str) -> anyhow::Result<String> {
    let manual = Manual::fetch(command, width)?;
    if manual.is_empty() {
        Manual::fetch(&command.to_lowercase(), width)
    } else {
        Ok(manual)
    }
}
