use crate::login;
use crate::parse;
use crate::user_config;
use log::{info,error};

async fn upload_helper(
    problem_id: &str,
    source: &str,
    ssid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Origin", "https://www.pbinfo.ro".parse()?);
    headers.insert(
        "Referer",
        format!("https://www.pbinfo.ro/probleme/{problem_id}").parse()?,
    );
    headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let form = reqwest::multipart::Form::new()
        .text("limbaj_de_programare", "cpp")
        .text("sursa", source.to_string())
        .text("id", problem_id.to_string());

    let request = client
        .request(
            reqwest::Method::POST,
            "https://www.pbinfo.ro/ajx-module/php-solutie-incarcare.php",
        )
        .headers(headers)
        .multipart(form);

    let response = request.send().await?;
    let body = response.text().await?;

    //println!("{}", &body);

    Ok(body)
}

#[derive(thiserror::Error, Debug)]
enum UploadError {
    #[error("Error: The user is not logged in!\nThe password or the email may be incorect!")]
    NotLoggedIn,
}
async fn try_upload(
    id: &str,
    source: &str,
    ssid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = match upload_helper(id, source, ssid).await {
        Ok(val) => val,
        Err(_) => {
            error!("Could not connect to pbinfo!\n\tCheck network connection and that pbinfo dns is up.");
            std::process::exit(1);
        }
    };

    match parse::parse_response(&response) {
        Ok(val) => {
            info!("Upload succefull!");
            return Ok(val);
        }
        Err(err) => match err {
            parse::ResponseType::ParseError => {
                error!("Didn't get a valid response: {}", response);
                std::process::exit(1);
            }
            parse::ResponseType::UnknownUploadError => {
                error!("INVALID RESPONSE:{}", response);
            }
            parse::ResponseType::LipsaAuth => {
                info!("The user is not logged in!");
                return Err(Box::new(UploadError::NotLoggedIn));
            }
        },
    }

    Ok("ID".to_string())
}

/// Returns the score of a given solution
///
/// # Arguments
///
/// * `sol_id` - id of the solution to get the score of
/// * `ssid` - ssid of the user
pub async fn get_score(sol_id: &str, ssid: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Cookie", format!("SSID={ssid}").parse()?);
    let request = client
        .request(
            reqwest::Method::POST,
            format!(
                "https://www.pbinfo.ro/ajx-module/ajx-solutie-detalii-evaluare.php?force_reload&id={sol_id}"
            ),
        )
        .headers(headers);

    let response = request.send().await?;
    let text = response.text().await?;

    Ok(text)
}

/// Uploads a source and returns a **solution** id
///
/// # Arguments
///
/// * `user_config` - configuration that holds the holds login info about a user if ssid or form are invalid this function will attempt to log in and will replace them
/// * `source` - the source to be uploaded for evaluation
/// * `problem_id` - the id of the problem
pub async fn upload(
    user_config: &mut user_config::UserConfig,
    source: &str,
    problem_id: &str,
) -> String {
    match try_upload(problem_id, &source, &user_config.ssid).await {
        Ok(val) => val,
        Err(_) => {
            info!("Attempting to login!");
            match login::login(
                &user_config.email,
                &user_config.password,
                &mut user_config.form_token,
                &mut user_config.ssid,
            )
            .await
            {
                Ok(val) => {
                    user_config::save_config(&user_config);
                    val
                }
                Err(err) => {
                    error!("COULD NOT LOGIN: \n{err}");
                    std::process::exit(1);
                }
            };
            info!("Login succesfull!");
            match try_upload(problem_id, &source, &user_config.ssid).await {
                Ok(val) => val,
                Err(_) => {
                    error!("The password or the email may be incorect, please double check!");
                    std::process::exit(1);
                }
            }
        }
    }
}
