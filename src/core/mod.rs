use lister::ManLister;

mod lister;

pub fn test() -> anyhow::Result<()> {
    let commands = ManLister::user_commands(None)?;
    for command in commands {
        println!("{command:?}");
    }

    Ok(())
}
