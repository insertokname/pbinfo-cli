use clap::Parser;

mod args;
mod login;
mod parse;
mod solution;
mod user_config;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let mut user_config = user_config::get_config();

    args::make_user_config(&mut user_config, &args);

    let source = std::fs::read_to_string(args.filename).expect("Could not read source file!\n");
    if source.is_empty() {
        println!("Given source file was empty!");
        std::process::exit(1);
    }

    println!("Uploading solution...");
    let solution_id = solution::upload(&mut user_config, &source, &args.id_problema).await;

    println!("Program is being evaluated!");
    while let Err(err) = parse::parse_score(
        &solution::get_score(&solution_id, &user_config.ssid)
            .await
            .unwrap(),
    ) {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        if let Some(down_err) = err.downcast_ref::<parse::ParseError>() {
            if *down_err == parse::ParseError::StillExecuting {
                println!("Program is still being evaluated...!");
                continue;
            }
        }
        break;
    }
}
