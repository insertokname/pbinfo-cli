use std::fs;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct UserConfig {
    pub email: String,
    pub password: String,
    pub ssid: String,
    pub form_token: String,
}

impl UserConfig {
    pub fn default() -> UserConfig {
        UserConfig {
            email: "".to_string(),
            password: "".to_string(),
            ssid: "".to_string(),
            form_token: "".to_string(),
        }
    }
}

/// Saves `config` in local storage
pub fn save_config(config: &UserConfig) {
    let proj_dirs = directories::ProjectDirs::from("dev", "insertokername", "pbinfo-cli").unwrap();
    let config_dir = proj_dirs.config_dir();

    let file_path = config_dir.join("pbinfo.toml");
    if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)
                .expect("could not create config parent folders!\nCheck permisions!");
        }
    }

    let _ = std::fs::File::create(&file_path).expect("could not create file\nCheck permisions!");

    std::fs::write(file_path, toml::to_string(&config).unwrap())
        .expect("could not write config file!");
}

/// Returns a locally stored config
pub fn get_config() -> UserConfig {
    let proj_dirs = directories::ProjectDirs::from("dev", "insertokername", "pbinfo-cli").unwrap();
    let config_dir = proj_dirs.config_dir();
    // println!("{:#?}",config_dir);

    let config_file = fs::read_to_string(config_dir.join("pbinfo.toml"));

    let config: UserConfig = match config_file {
        Ok(file) => toml::from_str(&file).unwrap(),
        Err(_) => {
            save_config(&UserConfig::default());
            UserConfig::default()
        }
    };
    config
}
