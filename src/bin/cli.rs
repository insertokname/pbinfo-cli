use clap::Parser;
use log::{error, info};
use pbinfo_api::pbinfo_user::PbinfoUser;
use pbinfo_cli::display::{self, ask_user_credentials};

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

    let source = std::fs::read_to_string(args.filename).expect("Could not read source file!\n");
    if source.is_empty() {
        error!("Given source file was empty!");
        std::process::exit(1);
    }

    let mut pbinfo_user = PbinfoUser::get_config().unwrap_or_else(|_| ask_user_credentials());

    log::info!("Logging in...");
    pbinfo_user.login().await?;
    pbinfo_user.save_config()?;

    info!("Uploading solution...");
    let solution_id = pbinfo_user.upload(&args.problem_id, &source).await?;

    info!("Program is being evaluated!");
    let score = pbinfo_user.pool_score(&solution_id).await?;

    display::display_score(score)?;
    Ok(())
}
