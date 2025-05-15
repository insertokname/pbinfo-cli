use clap::Parser;
use pbinfo_api::pbinfo_user::{PbinfoUser, TopSolutionResponseType};
use pbinfo_cli::display::ask_user_credentials;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Write};
use thiserror::Error;

mod bot_mod;
use bot_mod::{args, solve_repeated};

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

    let mut pbinfo_user = PbinfoUser::get_config().unwrap_or_else(|_| ask_user_credentials());

    log::info!("Logging in...");
    pbinfo_user.save_config()?;
    pbinfo_user.login().await?;
    pbinfo_user.save_config()?;

    // pub async fn solve(problem_id: &str, pbinfo_user: &PbinfoUser) -> Result<String, SolveError> {
    //     let correct_solution =
    //         get_raw_solution(problem_id)
    //             .await
    //             .map_err(|err| SolveError::GetSolutionError {
    //                 problem_id: problem_id.to_string(),
    //                 err: err.to_string(),
    //             })?;

    //     loop {
    //         match upload(&problem_id, &correct_solution, pbinfo_user).await {
    //             Ok(ok) => return Ok(ok),
    //             Err(err) => match err {
    //                 UploadError::CooldownError => {
    //                     tokio::time::sleep(Duration::from_secs(11)).await;
    //                 }
    //                 err => {
    //                     return Err(SolveError::UploadError {
    //                         problem_id: problem_id.to_string(),
    //                         err: err,
    //                     })
    //                 }
    //             },
    //         }
    //     }
    // }

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
            let solution_type = pbinfo_user.get_top_score(&problem_id).await;
            match solution_type {
                TopSolutionResponseType::PageError(err) => {
                    log::error!(
                        "Got error while getting your solutions for problem {i}!\nError was: {err}"
                    );
                }
                TopSolutionResponseType::ImperfectSolution
                | TopSolutionResponseType::NoSolution
                    if !args.dry_run =>
                {
                    match solve_repeated(&mut pbinfo_user, &problem_id).await {
                        Ok(ok) => {
                            solutions.insert(problem_id, ok);
                        }
                        Err(err) => {
                            log::error!("Got error while solving a problem {i}!\nError was: {err}");
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(args.delay)).await;
                }
                solution_type => {
                    solutions.insert(problem_id, solution_type);
                }
            };
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
