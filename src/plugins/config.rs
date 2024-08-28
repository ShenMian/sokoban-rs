use std::{fs, path::Path};

use crate::resources::{Config, PlayerMovement};

use bevy::prelude::*;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        if !Path::new(CONFIG_FILE_PATH).is_file() {
            let default_config_toml = toml::to_string(&Config::default()).unwrap();
            fs::write(CONFIG_FILE_PATH, default_config_toml).unwrap();
            save_config(&Config::default());
        }

        let config = load_config();
        let player_movement = PlayerMovement::new(config.player_move_speed);
        app.insert_resource(config).insert_resource(player_movement);

        app.add_systems(
            Update,
            save_config_system.run_if(resource_changed_or_removed::<Config>()),
        );
    }
}

const CONFIG_FILE_PATH: &str = "config.toml";

fn load_config() -> Config {
    let config_toml = fs::read_to_string(CONFIG_FILE_PATH).unwrap();
    let config: Config = toml::from_str(config_toml.as_str()).unwrap();
    config
}

fn save_config(config: &Config) {
    let config_toml = toml::to_string(&config).unwrap();
    fs::write(CONFIG_FILE_PATH, config_toml).unwrap();
}

fn save_config_system(config: Res<Config>) {
    save_config(&config);
}
