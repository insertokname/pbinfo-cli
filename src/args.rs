use clap::Parser;

use crate::user_config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the source file
    #[arg(short, long, default_value_t={"main.cpp".to_string()})]
    pub filename: String,

    /// If set asks for a new email when run
    #[arg(long)]
    pub reset_email: bool,

    /// If set asks for a new password when run
    #[arg(long)]
    pub reset_password: bool,

    #[arg(short, long)]
    pub problem_id: String,

    /// If set prints out more debuging info
    #[arg(short, long)]
    pub verbose: bool,
}

/// Return through the user_config argument a user_config with a password and email
///
/// # Arguments
///
/// * `local_user_config` - user_config that will hold the password and email
/// *  `args` - arguments that decide if the user password should be reset or loaded from local storage or any other option
pub fn make_user_config(local_user_config: &mut user_config::UserConfig, args: &Args) {
    if local_user_config.email == "" || args.reset_email {
        println!("Enter email:");
        local_user_config.email.clear();
        std::io::stdin()
            .read_line(&mut local_user_config.email)
            .expect("invalid email!");

        local_user_config.email = local_user_config.email.trim().to_string();
        user_config::save_config(&local_user_config);
    }

    if local_user_config.password == "" || args.reset_password {
        println!("Enter password:");
        local_user_config.password.clear();
        std::io::stdin()
            .read_line(&mut local_user_config.password)
            .expect("invalid password!");
        local_user_config.password = local_user_config.password.trim().to_string();
        user_config::save_config(&local_user_config);
    };
}