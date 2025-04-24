use std::time::Duration;

use reqwest::StatusCode;

use crate::api;

use super::UploadError;

#[derive(thiserror::Error, Debug)]
enum GetSolutionError {
    #[error("Couldn't find a solution for the problem {problem_id} on github codulluiandrei")]
    NoGithubSolution { problem_id: String },
    #[error("Couldn't create a reqwest client!\nGot error {err}")]
    CreateReqwestClientError { err: String },
    #[error("Couldn't send a request to the url: '{url}'\nGot error {err}")]
    SendRequestError { err: String, url: String },
    #[error("Couldn't parse the text in a response from url: '{url}'\nGot error {err}")]
    RequestParseTextError { err: String, url: String },
}

async fn get_raw_solution(problem_id: &str) -> Result<String, GetSolutionError> {
    let client = reqwest::Client::builder().build().map_err(|err| {
        GetSolutionError::CreateReqwestClientError {
            err: err.to_string(),
        }
    })?;

    let url = format!("https://raw.githubusercontent.com/codulluiandrei/pbinfo/refs/heads/main/pbinfo-{problem_id}/main.cpp");
    let response = client
        .request(reqwest::Method::GET, &url)
        .send()
        .await
        .map_err(|err| GetSolutionError::SendRequestError {
            err: err.to_string(),
            url: url.clone(),
        })?;

    if response.status() != StatusCode::OK {
        return Err(GetSolutionError::NoGithubSolution {
            problem_id: problem_id.to_string(),
        });
    }

    let text = response
        .text()
        .await
        .map_err(|err| GetSolutionError::RequestParseTextError {
            err: err.to_string(),
            url,
        })?;
    Ok(text)
}

#[derive(thiserror::Error, Debug)]
pub enum SolveError {
    #[error("Couldn't get a solution for the problem {problem_id}\nGot error{err}")]
    GetSolutionError { problem_id: String, err: String },
    #[error("Couldn't upload a solution for the problem {problem_id}\nGot error{}",err.to_string())]
    UploadError {
        problem_id: String,
        err: UploadError,
    },
}

pub async fn solve(problem_id: &str, ssid: &str) -> Result<String, SolveError> {
    let correct_solution =
        get_raw_solution(problem_id)
            .await
            .map_err(|err| SolveError::GetSolutionError {
                problem_id: problem_id.to_string(),
                err: err.to_string(),
            })?;


    loop {
        match api::upload(&problem_id, &correct_solution, &ssid).await {
            Ok(ok) => return Ok(ok),
            Err(err) => match err {
                UploadError::CooldownError => {
                    log::error!("Too many solutions uploaded in too short of a time!\nWaiting 11 seconds and trying again");
                    tokio::time::sleep(Duration::from_secs(11)).await;
                }
                err => {
                    return Err(SolveError::UploadError {
                        problem_id: problem_id.to_string(),
                        err: err,
                    })
                }
            },
        }
    }
}
