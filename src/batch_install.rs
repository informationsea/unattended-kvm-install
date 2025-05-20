use anyhow::Context;
use clap::{Args, Parser};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstall {
    #[arg(long, help = "Option CSV file path")]
    csv_options: String,
    #[arg(long, help = "Global options text file path")]
    global_options: String,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct BatchInstallParser {
    #[command(flatten)]
    run_all: crate::runall::RunAll,
}

impl BatchInstall {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut global_options = Vec::new();
        let global_options_reader = BufReader::new(
            File::open(&self.global_options).context("Failed to open global options file")?,
        );
        for l in global_options_reader.lines() {
            let l = l?;
            global_options.push(l.trim().to_string());
        }

        let csv_options_reader = BufReader::new(
            File::open(&self.csv_options).context("Failed to open CSV options file")?,
        );
        for mut row in crate::options_from_csv::generate_options_from_csv(csv_options_reader)? {
            let mut cmd = global_options.clone();
            cmd.append(&mut row);
            let cli = BatchInstallParser::parse_from(cmd.iter());
            eprintln!("#### Creating {} ####", cli.run_all.create_vm.vm_name);
            cli.run_all.run()?;
        }

        Ok(())
    }
}
