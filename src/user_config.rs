use crate::args;
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

    /// Saves `config` in local storage
    pub fn save_config(&self) {
        let proj_dirs =
            directories::ProjectDirs::from("dev", "insertokername", "pbinfo-cli").unwrap();
        let config_dir = proj_dirs.config_dir();

        let file_path = config_dir.join("pbinfo.toml");
        if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)
                    .expect("could not create config parent folders!\nCheck permisions!");
            }
        }

        let _ =
            std::fs::File::create(&file_path).expect("could not create file\nCheck permisions!");

        std::fs::write(file_path, toml::to_string(self).unwrap())
            .expect("could not write config file!");
    }

    /// Returns a locally stored config
    pub fn get_config() -> UserConfig {
        let proj_dirs =
            directories::ProjectDirs::from("dev", "insertokername", "pbinfo-cli").unwrap();
        let config_dir = proj_dirs.config_dir();
        // println!("{:#?}",config_dir);

        let config_file = fs::read_to_string(config_dir.join("pbinfo.toml"));

        let config: UserConfig = match config_file {
            Ok(file) => toml::from_str(&file).unwrap(),
            Err(_) => {
                Self::save_config(&UserConfig::default());
                UserConfig::default()
            }
        };
        config
    }

    /// Return through the user_config argument a user_config with a password and email
    ///
    /// # Arguments
    ///
    /// * `local_user_config` - user_config that will hold the password and email
    /// *  `args` - arguments that decide if the user password should be reset or loaded from local storage or any other option
    pub fn make_user_config(&mut self, args: &args::Args) {
        if self.email == "" || args.reset_email {
            println!("Enter email:");
            self.email.clear();
            std::io::stdin()
                .read_line(&mut self.email)
                .expect("invalid email!");

            self.email = self.email.trim().to_string();
            Self::save_config(&self);
        }

        if self.password == "" || args.reset_password {
            println!("Enter password:");
            self.password.clear();
            std::io::stdin()
                .read_line(&mut self.password)
                .expect("invalid password!");
            self.password = self.password.trim().to_string();
            Self::save_config(&self);
        };
    }
}
