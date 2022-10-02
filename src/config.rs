use std::fs::{create_dir_all, File, OpenOptions, read_dir, read_to_string};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use anyhow::{anyhow, Error};
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use whoami::username;

type Result<T> = std::result::Result<T, Error>;

pub static WORKING_DIR: Lazy<String> = Lazy::new(|| {
    format!("/home/{}/.local/share/chat_analyser/", username())
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub main_win_config: MainWindowConfig,
}

impl ConfigFile {
    pub fn new() -> Result<ConfigFile> {
        log::debug!("Checking for config file");
        match File::open(format!("{}config.json", *WORKING_DIR)) {
            Ok(_) => {
                log::debug!("Config file found at {}config.json", *WORKING_DIR);
                Ok(Self {
                    main_win_config: MainWindowConfig::new()?,
                })
            },
            Err(_) => {
                log::warn!("Failed to open config file! Using default settings!");
                Ok(Self::default())
            },
        }
    }

    pub fn create_folders() {
        for i in ["cache", "logs"] {
            log::debug!("Checking for {} folder", i);
            let path = format!("{}{}", *WORKING_DIR, i);
            if let Err(e) = read_dir(&path) {
                if e.to_string().to_lowercase().contains("no such file or directory") {
                    create_dir_all(path).unwrap();
                    log::debug!("Created {} folder", i);
                } else {
                    panic!("Failed to check for config directory at {:?} {}", path, e)
                }
            }
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        let default = Self {
            main_win_config: Default::default(),
        };
        match OpenOptions::new().create(true).write(true).truncate(true).open(format!("{}config.json", *WORKING_DIR)) {
            Ok(default_conf_file) => {
                if let Err(e) = serde_json::to_writer_pretty(default_conf_file, &default) {
                    panic!("Failed to create config file: {:?}", e);
                } else { log::info!("Created init config file at {}config.json", *WORKING_DIR); }
            }
            Err(e) => panic!("Failed to create config file: {:?}", e),
        }
        Self {
            main_win_config: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainWindowConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub window_position_x: f32,
    pub window_position_y: f32,
    pub dark_mode: bool,
}

impl MainWindowConfig {
    pub fn new() -> Result<MainWindowConfig> {
        match File::open(format!("{}config.json", *WORKING_DIR)) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader::<_, ConfigFile>(reader) {
                    Ok(window) => Ok(window.main_win_config),
                    Err(e) => {
                        log::error!("Failed to load user configuration: [{}] Using default settings!", e);
                        Ok(MainWindowConfig::default())
                    },
                }
            }
            Err(_) => {
                log::warn!("Existing window config data not found! Using default");
                Ok(MainWindowConfig::default())
            },
        }
    }

    pub fn save_window_to_json(window: eframe::WindowInfo, dark_mode: bool) {
        // Read config file into memery
        match read_to_string(format!("{}config.json", *WORKING_DIR)) {
            Ok(data) => {
                match serde_json::from_str(&data) {
                    Ok(file2) => {
                        let file2: Value = file2;
                        let mut i = file2.as_object().unwrap().clone();
                        *i["main_win_config"].get_mut("dark_mode").unwrap() = Value::from(dark_mode);
                        *i["main_win_config"].get_mut("window_width").unwrap() = Value::from(window.size.x);
                        *i["main_win_config"].get_mut("window_height").unwrap() = Value::from(window.size.y);
                        *i["main_win_config"].get_mut("window_position_x").unwrap() = Value::from(window.position.unwrap().x);
                        *i["main_win_config"].get_mut("window_position_y").unwrap() = Value::from(window.position.unwrap().y);

                        // Once config file is parsed and new values are written (in memory), wipe the file clean and then write onto disk
                        match OpenOptions::new().read(true).create(true).write(true).append(false).truncate(true).open(format!("{}config.json", *WORKING_DIR)) {
                            Ok(file) => {
                                if let Err(e) = serde_json::to_writer_pretty(file, &i) {
                                    log::error!("{:?}", e);
                                } else {
                                    log::info!("Saved window state")
                                }
                            }
                            Err(e) => log::error!("Failed to write window data to config: {}", e),
                        }
                    }
                    Err(e) => log::error!("Failed to parse config file: {}", e),
                }
            }
            Err(e) => log::error!("Failed to write window data to config: {}", e),
        }
    }
}

impl Default for MainWindowConfig {
    fn default() -> Self {
        Self {
            window_width: 350.0,
            window_height: 720.0,
            window_position_x: 0.0,
            window_position_y: 0.0,
            dark_mode: true,
        }
    }
}
