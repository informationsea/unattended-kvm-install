mod batch_install;
mod createvm;
mod kickstart;
mod options_from_csv;
mod passwd;
mod runall;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    #[command(about = "Create a VM using virt-install")]
    CreateVm(createvm::CreateVm),
    #[command(about = "Create a kickstart file")]
    Kickstart(kickstart::Kickstart),
    #[command(about = "Encrypt a password")]
    EncryptPasswd(passwd::Passwd),
    #[command(about = "Create a kickstart file and create a VM")]
    RunAll(runall::RunAll),
    #[command(about = "Batch install VMs using a CSV file")]
    BatchInstall(batch_install::BatchInstall),
}

#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::EncryptPasswd(x) => x.run()?,
        Command::Kickstart(x) => x.run()?,
        Command::CreateVm(x) => x.run()?,
        Command::RunAll(x) => x.run()?,
        Command::BatchInstall(x) => x.run()?,
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    #[test]
    fn create_config() {
        let file = File::create("target/config.json").expect("Failed to open config");
        let cli = Cli::parse_from(
            [
                "main",
                "run-all",
                "--iso",
                "ISO-File",
                "--vm-name",
                "VM-NAME",
            ]
            .iter(),
        );
        serde_json::to_writer_pretty(file, &cli).expect("Failed to write config");
    }
}
