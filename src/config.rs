use std::collections::HashMap;

use read_input::{prelude::input, InputBuild};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::error::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Settings {
    pub belabox: Belabox,
    pub twitch: Twitch,
    pub commands: HashMap<BotCommand, CommandInformation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Belabox {
    pub remote_key: String,
    pub custom_interface_name: HashMap<String, String>,
    pub monitor: Monitor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Monitor {
    pub modems: bool,
    pub notifications: bool,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            modems: true,
            notifications: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Twitch {
    pub bot_username: String,
    pub bot_oauth: String,
    pub channel: String,
    pub admins: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandInformation {
    pub command: String,
    pub permission: Permission,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum BotCommand {
    Bitrate,
    Network,
    Poweroff,
    Restart,
    Sensor,
    Start,
    Stats,
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Permission {
    Broadcaster,
    Moderator,
    Vip,
    Public,
}

impl Settings {
    /// Loads the config
    pub fn load<P>(path: P) -> Result<Self, ConfigError>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::read_to_string(path)?;
        let mut config = match serde_json::from_str::<Settings>(&file) {
            Ok(c) => c,
            Err(e) => {
                error!(%e, "config error");
                return Err(ConfigError::Json(e));
            }
        };

        // Lowercase important settings such as the twitch channel name to
        // avoid issues.
        lowercase_settings(&mut config);

        // Insert chat commands in the config if they don't exist.
        default_chat_commands(&mut config.commands);

        std::fs::write(CONFIG_FILE_NAME, serde_json::to_string_pretty(&config)?)?;

        Ok(config)
    }

    pub async fn ask_for_settings() -> Result<Self, ConfigError> {
        println!("Please paste your BELABOX Cloud remote URL below");

        let remote_key: String = input().msg("URL: ").get();
        let remote_key = remote_key.split("?key=").nth(1).expect("No key found");

        let mut custom_interface_name = HashMap::new();
        custom_interface_name.insert("eth0".to_string(), "eth0".to_string());
        custom_interface_name.insert("usb0".to_string(), "usb0".to_string());
        custom_interface_name.insert("wlan0".to_string(), "wlan0".to_string());

        let belabox = Belabox {
            remote_key: remote_key.to_string(),
            custom_interface_name,
            monitor: Monitor::default(),
        };

        println!("\nPlease enter your Twitch details below");
        let twitch = Twitch {
            bot_username: input().msg("Bot username: ").get(),
            bot_oauth: input()
                .msg("(You can generate an Oauth here: https://twitchapps.com/tmi/)\nBot oauth: ")
                .get(),
            channel: input().msg("Channel name: ").get(),
            admins: Vec::new(),
        };

        let mut commands = HashMap::new();
        default_chat_commands(&mut commands);

        let mut settings = Self {
            belabox,
            twitch,
            commands,
        };

        std::fs::write(CONFIG_FILE_NAME, serde_json::to_string_pretty(&settings)?)?;

        // FIXME: Does not work on windows
        print!("\x1B[2J");

        let mut path = std::env::current_dir()?;
        path.push(CONFIG_FILE_NAME);
        println!(
            "Saved settings to {} in {}",
            CONFIG_FILE_NAME,
            path.display()
        );

        lowercase_settings(&mut settings);

        Ok(settings)
    }
}

/// Lowercase settings which should always be lowercase
fn lowercase_settings(settings: &mut Settings) {
    let Twitch {
        bot_username,
        bot_oauth,
        channel,
        admins,
        ..
    } = &mut settings.twitch;

    *channel = channel.to_lowercase();
    *bot_oauth = bot_oauth.to_lowercase();
    *bot_username = bot_username.to_lowercase();

    for user in admins {
        *user = user.to_lowercase();
    }

    for info in settings.commands.values_mut() {
        info.command = info.command.to_lowercase();
    }
}

// Insert default commands if they don't exist
fn default_chat_commands(commands: &mut HashMap<BotCommand, CommandInformation>) {
    commands
        .entry(BotCommand::Start)
        .or_insert(CommandInformation {
            command: "!bbstart".to_string(),
            permission: Permission::Broadcaster,
        });

    commands
        .entry(BotCommand::Stop)
        .or_insert(CommandInformation {
            command: "!bbstop".to_string(),
            permission: Permission::Broadcaster,
        });

    commands
        .entry(BotCommand::Stats)
        .or_insert(CommandInformation {
            command: "!bbs".to_string(),
            permission: Permission::Public,
        });

    commands
        .entry(BotCommand::Restart)
        .or_insert(CommandInformation {
            command: "!bbrs".to_string(),
            permission: Permission::Broadcaster,
        });

    commands
        .entry(BotCommand::Poweroff)
        .or_insert(CommandInformation {
            command: "!bbpo".to_string(),
            permission: Permission::Broadcaster,
        });

    commands
        .entry(BotCommand::Bitrate)
        .or_insert(CommandInformation {
            command: "!bbb".to_string(),
            permission: Permission::Broadcaster,
        });

    commands
        .entry(BotCommand::Sensor)
        .or_insert(CommandInformation {
            command: "!bbsensor".to_string(),
            permission: Permission::Public,
        });

    commands
        .entry(BotCommand::Network)
        .or_insert(CommandInformation {
            command: "!bbt".to_string(),
            permission: Permission::Broadcaster,
        });
}