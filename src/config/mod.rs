use super::errors::*;

use std::env;
use std::path;

#[derive(Clone)]
pub struct Config {
    pub channels: Vec<String>,
    pub players: Vec<String>,
    pub chat_logs: path::PathBuf,
}

impl Config {
    pub fn default() -> Result<Config> {
        Ok({
            Config {
                channels: vec!["Local".to_string()],
                players: vec![],
                chat_logs: env::home_dir()
                    .chain_err(|| "chat log directory not found")?
                    .join("Documents/EVE/logs/Chatlogs"),
            }
        })
    }

    pub fn player(mut self, player: &str) -> Config {
        self.players.push(player.to_string());
        self
    }

    pub fn channel(mut self, channel: &str) -> Config {
        self.channels.push(channel.to_string());
        self
    }
}
