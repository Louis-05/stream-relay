use std::env::var;

use anyhow::{Context, bail};

pub struct Config {
    pub input_port: u16,
    pub output_port: u16,
    //pub stream_id: String,
    pub passphrase: String,
    pub web_port: u16,
}

impl Config {
    pub fn load_from_env() -> Result<Self, anyhow::Error> {
        let passphrase = var("PASSPHRASE").context("PASSPHRASE var not found")?;
        if passphrase.len() < 10 || passphrase.len() > 80 {
            bail!("Passphrase must be between 10 and 80 characters ")
        }

        Ok(Config {
            input_port: var("INPUT_PORT")
                .context("INPUT_PORT var not found")?
                .parse::<u16>()
                .context("Invalid port number")?,
            output_port: var("OUTPUT_PORT")
                .context("OUTPUT_PORT var not found")?
                .parse::<u16>()
                .context("Invalid port number")?,
            web_port: var("WEB_PORT")
                .context("WEB_PORT var not found")?
                .parse::<u16>()
                .context("Invalid port number")?,
            passphrase,
        })
    }
}
