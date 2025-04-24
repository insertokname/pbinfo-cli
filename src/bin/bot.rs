use clap::Parser;
use pbinfo_cli::api::score::TopSolutionResponseType;
use pbinfo_cli::{api, user_config};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use thiserror::Error;

mod bot_mod;
use bot_mod::args;

const CONSEQUENT_MAX_ERR_COUNT: u32 = 250;
const FOUND_PROBLEMS_FILENAME: &str = "found.json";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        log::error!("Got error while finding solutions:\n{}", err.to_string());
    };
}

#[derive(Error, Debug)]
#[error("start_problem_id must be smaller than end_problem_id!\nInstead got:\nstart_problem_id: {start_problem_id}\nend_problem_id: {end_problem_id}\nPlease pass the arguments correctly!")]
struct ArgParseError {
    start_problem_id: u32,
    end_problem_id: u32,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();
    if args.start_problem_id > args.end_problem_id {
        return Err(ArgParseError {
            start_problem_id: args.start_problem_id,
            end_problem_id: args.end_problem_id,
        }
        .into());
    }

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();

    if args.dry_run {
        log::info!("DRY RUN:\nThis run will not actually solve any problems but it will just check which problems you have solved and if they have 100 points or not!");
    }

    let file = File::open(FOUND_PROBLEMS_FILENAME).unwrap_or_else(|_| {
        let mut file = File::create(FOUND_PROBLEMS_FILENAME)
            .expect(format!("Couldn't create {FOUND_PROBLEMS_FILENAME} file!").as_str());
        file.write("{}".as_bytes())
            .expect(format!("Could not write to {FOUND_PROBLEMS_FILENAME} file!").as_str());
        File::open(FOUND_PROBLEMS_FILENAME).expect(
            format!("Couldn't open {FOUND_PROBLEMS_FILENAME} even after creating it!").as_str(),
        )
    });

    let reader = BufReader::new(file);
    let mut solutions: BTreeMap<String, TopSolutionResponseType> = serde_json::from_reader(reader)
        .expect(&format!(
            "Unable to parse found JSON {}",
            FOUND_PROBLEMS_FILENAME
        ));

    let mut user_config = user_config::UserConfig::get_config();
    user_config.update_user_config(args.reset_email, args.reset_password, args.reset_user_id);

    log::info!("Logging in...");
    api::login(
        &user_config.email,
        &user_config.password,
        &mut user_config.form_token,
        &mut user_config.ssid,
    )
    .await?;
    user_config.save_config();

    let mut consequent_err_count = 0;
    for i in args.start_problem_id..=args.end_problem_id {
        let problem_id = i.to_string();
        let parse_scores = match solutions.get(&problem_id) {
            Some(some) => match some {
                TopSolutionResponseType::PageError(_) => true,
                TopSolutionResponseType::ImperfectSolution => true,
                TopSolutionResponseType::NoSolution => true,
                _ => false,
            },
            None => true,
        };
        if parse_scores {
            log::info!("Getting your solutions for {i}...");
            let solution_type = api::score::get_top_score(
                &problem_id,
                &user_config.ssid,
                user_config.user_id.as_ref().unwrap(),
            )
            .await;
            match solution_type {
                TopSolutionResponseType::PageError(err) => {
                    log::error!(
                        "Got error while getting your solutions for problem {i}!\nError was: {err}"
                    );
                    consequent_err_count += 1;
                }
                TopSolutionResponseType::ImperfectSolution
                | TopSolutionResponseType::NoSolution
                    if !args.dry_run =>
                {
                    log::info!("Uploading correct solution for problem!");
                    match api::solve(&problem_id, &user_config.ssid).await {
                        Ok(ok) => {
                            if let Err(err) = api::score::pool_score(&ok, &user_config.ssid).await {
                                log::error!(
                                    "Got error while pooling for execution to finish for problem {i}!\nError was: {err}"
                                );
                                consequent_err_count += 1;
                            }
                            solutions.insert(problem_id, TopSolutionResponseType::PerfectSolution);
                        }
                        Err(err) => {
                            log::error!(
                            "Got error while uploading a solution for problem {i}!\nError was:\n{err}"
                        );
                            consequent_err_count += 1;
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(args.delay)).await;
                }
                solution_type => {
                    consequent_err_count = 0;
                    solutions.insert(problem_id, solution_type);
                }
            };
            if consequent_err_count == CONSEQUENT_MAX_ERR_COUNT {
                log::error!("Encountered too many errors in a row!\nExiting program!");
                break;
            }
            if i % 10 == 0 {
                let file = File::create(FOUND_PROBLEMS_FILENAME).expect("Unable to create file");
                serde_json::to_writer(file, &solutions).expect("Unable to write data");
            }
        }
    }

    let file = File::create(FOUND_PROBLEMS_FILENAME).expect("Unable to create file");
    serde_json::to_writer(file, &solutions).expect("Unable to write data");

    Ok(())
}
