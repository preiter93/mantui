use lister::Lister;
use reader::Reader;

mod lister;
mod reader;

pub(crate) fn load_section(section: String) -> anyhow::Result<Vec<String>> {
    Lister::list_section(section)
}

pub(crate) fn read_command(command: &str, width: &str) -> anyhow::Result<String> {
    let manual = Reader::read(command, width)?;
    if manual.is_empty() {
        Reader::read(&command.to_lowercase(), width)
    } else {
        Ok(manual)
    }
}
