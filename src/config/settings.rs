use std::collections::HashMap;
use std::path::PathBuf;

use color_eyre::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub vault: VaultConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub path: PathBuf,
    #[serde(default = "default_extension")]
    pub default_extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_tree_width")]
    pub tree_width: u16,
    #[serde(default = "default_true")]
    pub show_backlinks: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub theme_overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_editor")]
    pub external: String,
}

fn default_extension() -> String {
    "md".to_string()
}

fn default_tree_width() -> u16 {
    25
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "gruvbox-dark".to_string()
}

fn default_editor() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
}

impl Default for VaultConfig {
    fn default() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            path: home.join("notes"),
            default_extension: default_extension(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            tree_width: default_tree_width(),
            show_backlinks: default_true(),
            theme: default_theme(),
            theme_overrides: HashMap::new(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            external: default_editor(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault: VaultConfig::default(),
            ui: UiConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let contents = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, contents)?;
            Ok(config)
        }
    }

    fn config_path() -> PathBuf {
        ProjectDirs::from("com", "tui-jot", "tui-jot")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
