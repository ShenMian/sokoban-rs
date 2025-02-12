use std::{fs, path::Path};

use crate::resources::{Config, PlayerMovement};

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    if !Path::new(&*CONFIG_FILE_PATH).is_file() {
        let default_config_toml = toml::to_string(&Config::default()).unwrap();
        fs::write(&*CONFIG_FILE_PATH, default_config_toml).unwrap();
        save_config(&Config::default());
    }

    let config = load_config();
    let player_movement = PlayerMovement::new(config.player_move_speed);
    app.insert_resource(config).insert_resource(player_movement);

    app.add_systems(
        Update,
        save_config_system.run_if(resource_changed_or_removed::<Config>),
    );
}

static CONFIG_FILE_PATH: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let mut path = crate::settings::app_writeable_dir();
    path.push("config.toml");
    path.to_str().unwrap().to_string()
});

fn load_config() -> Config {
    let config_toml = fs::read_to_string(&*CONFIG_FILE_PATH).unwrap();
    let config: Config = toml::from_str(config_toml.as_str()).unwrap();
    config
}

fn save_config(config: &Config) {
    let config_toml = toml::to_string(&config).unwrap();
    fs::write(&*CONFIG_FILE_PATH, config_toml).unwrap();
}

fn save_config_system(config: Res<Config>) {
    save_config(&config);
}
