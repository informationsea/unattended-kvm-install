use std::fmt::Display;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkBootproto {
    Dhcp,
    Static,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct KickstartNetwork {
    #[arg(long, help = "Text mode install")]
    text: bool,
    #[arg(long, help = "Network device", default_value = "enp1s0")]
    network_device: String,
    #[arg(long, help = "Network Boot Protocol", default_value = "dhcp")]
    network_bootproto: NetworkBootproto,
    #[arg(
        long,
        help = "IP Address for manual setup",
        default_value = "192.168.100.2"
    )]
    network_ip: String,
    #[arg(
        long,
        help = "IP Netmask for manual setup",
        default_value = "255.255.255.0"
    )]
    network_netmask: String,
    #[arg(
        long,
        help = "Gateway for manual setup",
        default_value = "192.168.100.1"
    )]
    network_gateway: String,
    #[arg(
        long,
        help = "Nameserver for manual setup",
        default_value = "192.168.100.1"
    )]
    network_nameserver: String,
    #[arg(long, help = "Host name", default_value = "localhost.localdomain")]
    network_hostname: String,
}

impl KickstartNetwork {
    pub fn generate(&self) -> String {
        match self.network_bootproto {
            NetworkBootproto::Dhcp => {
                format!(
                    "network --bootproto=dhcp --device={} --hostname={} --ipv6=auto --activate",
                    self.network_device, self.network_hostname
                )
            }
            NetworkBootproto::Static => {
                format!(
                "network --bootproto=static --ip={} --netmask={} --gateway={} --device={} --nameserver={} --hostname={} --ipv6=auto --activate",
                self.network_ip, self.network_netmask, self.network_gateway, self.network_device, self.network_nameserver, self.network_hostname
            )
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallEnvironment {
    MinimalEnvironment,
    GraphicalServerEnvironment,
    ServerProductEnvironment,
    WorkstationProductEnvironment,
    VirtualizationHostEnvironment,
}

impl Display for InstallEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallEnvironment::MinimalEnvironment => {
                write!(f, "minimal-environment")
            }
            InstallEnvironment::GraphicalServerEnvironment => {
                write!(f, "graphical-server-environment")
            }
            InstallEnvironment::ServerProductEnvironment => write!(f, "server-product-environment"),
            InstallEnvironment::WorkstationProductEnvironment => {
                write!(f, "workstation-product-environment")
            }
            InstallEnvironment::VirtualizationHostEnvironment => {
                write!(f, "virtualization-host-environment")
            }
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct KickstartSystem {
    #[arg(long, help = "Timezone", default_value = "Asia/Tokyo")]
    timezone: String,
    #[arg(long, help = "Keyboard layout (Example: \"ja\")", default_value = "us")]
    keyboard: String,
    #[arg(
        long,
        help = "Language (Example: \"ja_JP.UTF-8\")",
        default_value = "en_US.UTF-8"
    )]
    language: String,
    #[arg(
        long,
        help = "Additional packages to install",
        default_value = "@standard\n@guest-agents"
    )]
    packages: Vec<String>,
    #[arg(
        long,
        help = "Install Environment",
        default_value = "minimal-environment"
    )]
    environment: InstallEnvironment,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct RootPw {
    #[arg(long, help = "Root plain password", conflicts_with_all = ["rootpw_crypt", "rootpw_keyboard", "rootpw_locked"])]
    rootpw_plain: Option<String>,
    #[arg(long, help = "Root crypt password", conflicts_with_all = ["rootpw_plain", "rootpw_keyboard", "rootpw_locked"])]
    rootpw_crypt: Option<String>,
    #[arg(
        long,
        help = "Root password from keyboard",
        conflicts_with_all = ["rootpw_plain", "rootpw_crypt", "rootpw_locked"]
    )]
    rootpw_keyboard: bool,
    #[arg(
        long,
        help = "Root user is locked", 
        conflicts_with_all = ["rootpw_plain", "rootpw_crypt", "rootpw_keyboard"]
    )]
    rootpw_locked: bool,
    #[arg(long, help = "Root sshkey")]
    root_sshkey: Option<String>,
}

impl RootPw {
    pub fn generate(&self) -> anyhow::Result<String> {
        let sshkey = if let Some(sshkey) = &self.root_sshkey {
            format!("\nsshkey --username=root \"{sshkey}\"")
        } else {
            "".to_string()
        };
        if let Some(pw) = &self.rootpw_plain {
            Ok(format!("rootpw --plaintext {pw}{sshkey}"))
        } else if let Some(pw) = &self.rootpw_crypt {
            Ok(format!("rootpw --iscrypted {pw}{sshkey}"))
        } else if self.rootpw_keyboard {
            let encrypt_password = crate::passwd::read_and_encrypt_password("Root Password: ")?;
            Ok(format!("rootpw --iscrypted {encrypt_password}{sshkey}"))
        } else if self.rootpw_locked {
            Ok("rootpw --lock".to_string())
        } else {
            Err(anyhow::anyhow!("Root password is not set"))
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    #[arg(long, help = "Storage device", default_value = "vda")]
    storage_device: String,
    #[arg(long, help = "Filesystem", default_value = "xfs")]
    filesystem: String,
}

impl Storage {
    pub fn generate(&self) -> String {
        let storage_device = &self.storage_device;
        let filesystem = &self.filesystem;
        format!(
            r#"ignoredisk --only-use={storage_device}
# Partition clearing information
clearpart --none --initlabel
# Disk partitioning information
reqpart
part pv.116 --fstype="lvmpv" --ondisk={storage_device} --size=15360 --grow
part /boot --fstype="{filesystem}" --ondisk={storage_device} --size=1024
volgroup almalinux --pesize=4096 pv.116
logvol swap --fstype="swap" --size=4030 --name=swap --vgname=almalinux
logvol / --fstype="{filesystem}" --size=10240 --name=root --vgname=almalinux --grow
"#
        )
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct UserPw {
    #[arg(long, help = "Username")]
    username: Option<String>,
    #[arg(long, help = "User plain password", conflicts_with_all = ["user_crypt", "user_keyboard"])]
    user_plain: Option<String>,
    #[arg(long, help = "User crypt password", conflicts_with_all = ["user_plain", "user_keyboard"])]
    user_crypt: Option<String>,
    #[arg(
        long,
        help = "User password from keyboard",
        conflicts_with_all = ["user_plain", "user_crypt"]
    )]
    user_keyboard: bool,
    #[arg(long, help = "User groups")]
    user_groups: Option<Vec<String>>,
    #[arg(long, help = "User sshkey")]
    user_sshkey: Option<String>,
    #[arg(long, help = "User UID")]
    user_uid: Option<u32>,
    #[arg(long, help = "User GID")]
    user_gid: Option<u32>,
}

impl UserPw {
    pub fn generate(&self) -> anyhow::Result<String> {
        if let Some(username) = &self.username {
            let sshkey = if let Some(sshkey) = &self.user_sshkey {
                format!("\nsshkey --username={username} \"{sshkey}\"")
            } else {
                "".to_string()
            };
            let pw = if let Some(pw) = &self.user_plain {
                format!("--password={pw} --plaintext")
            } else if let Some(pw) = &self.user_crypt {
                format!("--password={pw} --iscrypted")
            } else if self.user_keyboard {
                let encrypt_password = crate::passwd::read_and_encrypt_password("User Password: ")?;
                format!("--password={encrypt_password} --iscrypted")
            } else {
                return Err(anyhow::anyhow!("User password is not set"));
            };
            let groups = if let Some(groups) = self.user_groups.as_ref() {
                format!("--groups={}", groups.join(","))
            } else {
                "".to_string()
            };
            let uid = if let Some(uid) = &self.user_uid {
                format!("--uid={}", uid)
            } else {
                "".to_string()
            };
            let gid = if let Some(gid) = &self.user_gid {
                format!("--gid={}", gid)
            } else {
                "".to_string()
            };
            Ok(format!(
                "user --name={username} {pw} {groups} {uid} {gid}{sshkey}"
            ))
        } else {
            Ok("".to_string())
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct Kickstart {
    #[command(flatten, next_help_heading = "Kickstart Network")]
    #[serde(flatten)]
    network: KickstartNetwork,
    #[command(flatten, next_help_heading = "Kickstart System")]
    #[serde(flatten)]
    system_options: KickstartSystem,
    #[command(flatten, next_help_heading = "Kickstart Root Password")]
    #[serde(flatten)]
    rootpw: RootPw,
    #[command(flatten, next_help_heading = "Kickstart Storage")]
    #[serde(flatten)]
    storage: Storage,
    #[command(flatten, next_help_heading = "Kickstart User")]
    #[serde(flatten)]
    user: UserPw,
}

impl Kickstart {
    pub fn run(&self) -> anyhow::Result<()> {
        println!("{}", self.generate()?);
        Ok(())
    }

    pub fn generate(&self) -> anyhow::Result<String> {
        let network = self.network.generate();
        let rootpw = self.rootpw.generate()?;
        let storage = self.storage.generate();
        let user = self.user.generate()?;
        let install_mode = if self.network.text {
            "text"
        } else {
            "graphical"
        };
        let keyboard = &self.system_options.keyboard;
        let language = &self.system_options.language;
        let environment = &self.system_options.environment;
        let timezone = &self.system_options.timezone;
        let packages = self.system_options.packages.join("\n");

        if self.rootpw.rootpw_locked && self.user.username.is_none() {
            return Err(anyhow::anyhow!(
                "Root password is locked and user is not set"
            ));
        }
        if self.rootpw.rootpw_locked
            && !self
                .user
                .user_groups
                .as_ref()
                .map(|x| x.contains(&"wheel".to_string()))
                .unwrap_or(false)
        {
            return Err(anyhow::anyhow!(
                "Root password is locked and user is not in wheel group"
            ));
        }

        Ok(format!(
            r#"{install_mode}
eula --agreed
repo --name="AppStream" --baseurl=file:///run/install/sources/mount-0000-cdrom/AppStream
reboot

%addon com_redhat_kdump --enable --reserve-mb='auto'

%end

# Keyboard layouts
keyboard --xlayouts='{keyboard}'
# System language
lang {language}

# Network information
{network}

# Use CDROM installation media
cdrom

%packages
@^{environment}
{packages}

%end

# Run the Setup Agent on first boot
firstboot --enable

# Disk
{storage}

# System timezone
timezone {timezone} --utc

#Root password
{rootpw}
{user}

shutdown
"#
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_kickstart_network_dhcp() {
        let network = KickstartNetwork {
            text: false,
            network_device: "enp1s0".to_string(),
            network_bootproto: NetworkBootproto::Dhcp,
            network_ip: "10.0.0.2".to_string(),
            network_netmask: "255.255.0.0".to_string(),
            network_gateway: "10.0.0.254".to_string(),
            network_nameserver: "10.0.0.1".to_string(),
            network_hostname: "test.example.com".to_string(),
        };
        assert_eq!(
            network.generate(),
            r#"network --bootproto=dhcp --device=enp1s0 --hostname=test.example.com --ipv6=auto --activate"#
        );
    }

    #[test]
    fn test_kickstart_network_static() {
        let network = KickstartNetwork {
            text: false,
            network_device: "enp1s0".to_string(),
            network_bootproto: NetworkBootproto::Static,
            network_ip: "10.0.0.2".to_string(),
            network_netmask: "255.255.0.0".to_string(),
            network_gateway: "10.0.0.254".to_string(),
            network_nameserver: "10.0.0.1".to_string(),
            network_hostname: "test.example.com".to_string(),
        };
        assert_eq!(
            network.generate(),
            r#"network --bootproto=static --ip=10.0.0.2 --netmask=255.255.0.0 --gateway=10.0.0.254 --device=enp1s0 --nameserver=10.0.0.1 --hostname=test.example.com --ipv6=auto --activate"#
        );
    }

    #[test]
    fn test_kickstart_storage() {
        let storage = Storage {
            storage_device: "sda1".to_string(),
            filesystem: "ext4".to_string(),
        };
        assert_eq!(
            storage.generate(),
            r#"ignoredisk --only-use=sda1
# Partition clearing information
clearpart --none --initlabel
# Disk partitioning information
reqpart
part pv.116 --fstype="lvmpv" --ondisk=sda1 --size=15360 --grow
part /boot --fstype="ext4" --ondisk=sda1 --size=1024
volgroup almalinux --pesize=4096 pv.116
logvol swap --fstype="swap" --size=4030 --name=swap --vgname=almalinux
logvol / --fstype="ext4" --size=10240 --name=root --vgname=almalinux --grow
"#
        );
    }

    #[test]
    fn test_kickstart_rootpw_plain() {
        let rootpw = RootPw {
            rootpw_plain: Some("password".to_string()),
            rootpw_crypt: None,
            rootpw_keyboard: false,
            rootpw_locked: false,
            root_sshkey: Some("SSHKEY".to_string()),
        };
        assert_eq!(
            rootpw.generate().unwrap(),
            r#"rootpw --plaintext password
sshkey --username=root "SSHKEY""#
        );
    }

    #[test]
    fn test_kickstart_rootpw_crypt() {
        let rootpw = RootPw {
            rootpw_plain: None,
            rootpw_crypt: Some("CRYPT".to_string()),
            rootpw_keyboard: false,
            rootpw_locked: false,
            root_sshkey: None,
        };
        assert_eq!(rootpw.generate().unwrap(), r#"rootpw --iscrypted CRYPT"#);
    }

    #[test]
    fn test_kickstart_rootpw_lock() {
        let rootpw = RootPw {
            rootpw_plain: None,
            rootpw_crypt: None,
            rootpw_keyboard: false,
            rootpw_locked: true,
            root_sshkey: None,
        };
        assert_eq!(rootpw.generate().unwrap(), r#"rootpw --lock"#);
    }

    #[test]
    fn test_kickstart_user_plain() {
        let userpw = UserPw {
            username: Some("testuser".to_string()),
            user_plain: Some("password".to_string()),
            user_crypt: None,
            user_keyboard: false,
            user_uid: None,
            user_gid: None,
            user_groups: None,
            user_sshkey: None,
        };
        assert_eq!(
            userpw.generate().unwrap(),
            r#"user --name=testuser --password=password --plaintext   "#
        );
    }

    #[test]
    fn test_kickstart_user_crypt() {
        let userpw = UserPw {
            username: Some("testuser".to_string()),
            user_plain: None,
            user_crypt: Some("CRYPT".to_string()),
            user_keyboard: false,
            user_uid: Some(5000),
            user_gid: Some(6000),
            user_groups: Some(["wheel".to_string(), "docker".to_string()].to_vec()),
            user_sshkey: Some("SSHKEY".to_string()),
        };
        assert_eq!(
            userpw.generate().unwrap(),
            r#"user --name=testuser --password=CRYPT --iscrypted --groups=wheel,docker --uid=5000 --gid=6000
sshkey --username=testuser "SSHKEY""#
        );
    }
}
