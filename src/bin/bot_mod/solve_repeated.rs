use std::{
    fs::{self},
    sync::LazyLock,
    time::Duration,
};

use pbinfo_api::pbinfo_user::{PbinfoUser, SolveError, TopSolutionResponseType, UploadError};
use serde_json::Value;

pub static CUSTOM_SOLUTIONS: LazyLock<Option<Value>> = LazyLock::new(|| {
    let data = fs::read_to_string("solutions.json").ok()?;
    let maybe_json_content = serde_json::from_str(data.as_str());
    match maybe_json_content {
        Ok(ok) => Some(ok),
        Err(err) => {
            log::error!("Got error while parsing custom json solutions: {err}");
            None
        }
    }
});

pub async fn solve_repeated(
    pbinfo_user: &mut PbinfoUser,
    problem_id: &str,
) -> Result<TopSolutionResponseType, Box<dyn std::error::Error>> {
    log::info!("Uploading correct solution for problem!");
    let mut already_attempted_login: bool = false;
    loop {
        let res = match CUSTOM_SOLUTIONS.as_ref() {
            Some(some) => pbinfo_user.costume_solve(&problem_id, some).await,
            None => pbinfo_user.solve(problem_id).await,
        };
        match res {
            Ok(ok) => {
                pbinfo_user.pool_score(&ok).await?;
            }
            Err(SolveError::UploadError { problem_id, err }) => match err {
                UploadError::CooldownError => {
                    if already_attempted_login {
                        log::error!("Already reloged user!\n Waiting 5 minutes");
                        tokio::time::sleep(Duration::from_secs(60 * 5)).await;
                        continue;
                    }

                    log::info!("Refreshing user login to be able to upload more solutions!");
                    already_attempted_login = true;
                    if let Err(err) = pbinfo_user.fresh_login().await {
                        log::error!("Got error while refreshing user login!\nError was: {err}\nBot will wait for 5 minutes before uploading anything else!");
                        tokio::time::sleep(Duration::from_secs(60 * 5)).await;
                    }
                    continue;
                }
                UploadError::NotLoggedInError => {
                    log::error!("User wasn't logged in, attempting to log in again!");
                    pbinfo_user.fresh_login().await?;
                }
                err => return Err(SolveError::UploadError { problem_id, err }.into()),
            },
            Err(err) => {
                return Err(err.into());
            }
        };
        return Ok(pbinfo_user.get_top_score(problem_id).await);
    }
}
