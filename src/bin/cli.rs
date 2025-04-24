use clap::Parser;
use log::{error, info};
use pbinfo_cli::{
    api::{self, score},
    display, user_config,
};

mod cli_mod;
use cli_mod::args;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        log::error!("Got error while running:\n{}", err.to_string());
    };
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let mut user_config = user_config::UserConfig::get_config();

    user_config.update_user_config(args.reset_email, args.reset_password, args.reset_user_id);

    let source = std::fs::read_to_string(args.filename).expect("Could not read source file!\n");
    if source.is_empty() {
        error!("Given source file was empty!");
        std::process::exit(1);
    }

    api::login(
        &user_config.email,
        &user_config.password,
        &mut user_config.form_token,
        &mut user_config.ssid,
    )
    .await?;
    user_config.save_config();

    info!("Uploading solution...");
    let solution_id = api::upload(&args.problem_id, &source, &user_config.ssid).await?;


    info!("Program is being evaluated!");
    let score = score::pool_score(&solution_id, &user_config.ssid).await?;

    display::display_score(score)?;
    Ok(())
}
