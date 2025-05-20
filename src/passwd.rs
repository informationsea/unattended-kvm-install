use clap::Args;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use termion::input::TermRead;

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct Passwd {}

impl Passwd {
    pub fn run(&self) -> anyhow::Result<()> {
        let encrypt_password = read_and_encrypt_password("Password: ")?;
        println!("{}", encrypt_password);

        Ok(())
    }
}

pub fn read_passwd(prompt: &str) -> anyhow::Result<String> {
    let stdin = std::io::stdin();
    let mut stdin_locked = stdin.lock();
    let stderr = std::io::stderr();
    let mut stderr_locked = stderr.lock();
    stderr_locked.write_all(prompt.as_bytes())?;
    stderr_locked.flush()?;

    let password = stdin_locked.read_passwd(&mut stderr_locked)?;
    stderr_locked.write_all(b"\n")?;
    stderr_locked.flush()?;
    Ok(password.unwrap_or_default())
}

pub fn read_and_encrypt_password(prompt: &str) -> anyhow::Result<String> {
    let password = read_passwd(prompt)?;
    let password2 = read_passwd("Confirm: ")?;
    if password != password2 {
        return Err(anyhow::anyhow!("Passwords do not match"));
    }

    let params = sha_crypt::Sha512Params::new(5_000).expect("RandomError!");
    let encrypt_password = sha_crypt::sha512_simple(&password, &params).expect("Encrypt Error");
    Ok(encrypt_password)
}
