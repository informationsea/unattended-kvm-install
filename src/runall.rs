use std::io::Write;

use crate::createvm;
use crate::kickstart;

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct RunAll {
    #[command(flatten, next_help_heading = "Kickstart")]
    #[serde(flatten)]
    pub kickstart: kickstart::Kickstart,
    #[command(flatten, next_help_heading = "Create VM")]
    #[serde(flatten)]
    pub create_vm: createvm::CreateVmBase,
}

impl RunAll {
    pub fn run(&self) -> anyhow::Result<()> {
        let kickstart = self.kickstart.generate()?;
        let mut tmp = tempfile::NamedTempFile::new()?;
        tmp.write_all(kickstart.as_bytes())?;
        let kickstart_path = tmp.path().to_str().unwrap();
        self.create_vm.create_vm(Some(kickstart_path))?;

        Ok(())
    }
}
