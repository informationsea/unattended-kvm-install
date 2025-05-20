use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::Path;

fn disk_default() -> u32 {
    70
}

#[derive(Args, Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateVmBase {
    #[arg(long, short = 'n', help = "Virtual Machine Name")]
    pub vm_name: String,
    #[arg(long, help = "Virtual Machine Disk Size (GB)", default_value = "70")]
    #[serde(default = "disk_default")]
    disk_size: u32,
    #[arg(
        long,
        help = "Virtual Machine Memory Size (MB)",
        default_value = "4096"
    )]
    memory: u32,
    #[arg(long, help = "Virtual Machine # of CPUs", default_value = "2")]
    vcpu: u32,
    #[arg(
        long,
        help = "Network (example: bridge=br0)",
        default_value = "network=default,model=virtio"
    )]
    network: String,
    #[arg(long, help = "ISO image file path")]
    iso: String,
    #[arg(long, help = "Do not remove temporary directory after finish")]
    do_not_remove_temporary_directory: bool,
    #[arg(long, help = "Do not create VM but print virt-install command")]
    dry_run: bool,
    #[arg(
        long,
        help = "OS info (example: almalinux8)",
        default_value = "almalinux8"
    )]
    osinfo: String,
}

fn s(s: impl AsRef<str>) -> String {
    s.as_ref().to_string()
}

impl CreateVmBase {
    pub fn virt_install_cmd(&self, kickstart_path: Option<&str>) -> anyhow::Result<Vec<String>> {
        let disk = format!("size={}", self.disk_size);
        let vcpu = format!("{}", self.vcpu);
        let memory = format!("memory={0},maxmemory={0}", self.memory);
        let mut cmd = vec![
            s("virt-install"),
            s("--name"),
            s(&self.vm_name),
            s("--osinfo"),
            s(&self.osinfo),
            s("--disk"),
            s(disk),
            s("--vcpu"),
            s(&vcpu),
            s("--cpu"),
            s("host"),
            s("--memory"),
            s(&memory),
            s("--location"),
            s(&self.iso),
            s("--network"),
            s(&self.network),
            s("--noreboot"),
            s("--autoconsole"),
            s("text"),
        ];

        if let Some(kickstart_path) = kickstart_path {
            cmd.push(s("--initrd-inject"));
            cmd.push(s(kickstart_path));
            cmd.push(s("--extra-args"));
            cmd.push(format!(
                "inst.text  inst.ks=file:{}  console=ttyS0",
                Path::new(kickstart_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ));
        }

        Ok(cmd)
    }

    pub fn create_vm(&self, kickstart_path: Option<&str>) -> anyhow::Result<()> {
        let cmd = self.virt_install_cmd(kickstart_path)?;
        println!("\"{}\"", cmd.join("\" \""));
        if !self.dry_run {
            let status = std::process::Command::new("sudo")
                .args(cmd)
                .status()
                .expect("failed to execute process");
            assert!(status.success());
        }
        Ok(())
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct CreateVm {
    #[command(flatten)]
    base: CreateVmBase,
    #[arg(long, help = "Kickstart file path")]
    kickstart: Option<String>,
}

impl CreateVm {
    pub fn run(&self) -> anyhow::Result<()> {
        self.base.create_vm(self.kickstart.as_deref())?;
        Ok(())
    }
}
