use log::{error, info};
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseIdError {
    #[error("Error: An unknown upload error happened!\nResponse from pbinfo was: {response}")]
    UnknownUploadError { response: String },
    #[error("Error: Couldn't parse a response json:\n{json}\nError was: {err}\nTry again OR check if pbinfo is up by going to the website manually!")]
    ParseError { json: String, err: String },
    #[error("Error: The user wasn't logged in!")]
    NotLoggedInError,
    #[error("Error: Another solution is being evaulated!\nPlease wait for it to finish!")]
    AlreadyEvaluationError,
    #[error("Error: Too many solutions were uploaded in too short of a time!")]
    CooldownError,
}

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("Error: An error occurred when uploading the solution!\nError was: {err}")]
    UploadError { err: String },
    #[error("Error: There was an error when getting the id of the solution!\nError was: {err}")]
    ResponseParseError { err: String },
    #[error("Error: The user wasn't logged in!")]
    NotLoggedInError,
    #[error("Error: Got a status code that wasn't 200 when uploading solution!\nInstead got {status_code}")]
    BadStatusCodeError { status_code: String },
    #[error("Error: Too many solutions were uploaded in too short of a time!")]
    CooldownError,
}

#[derive(Debug, Error)]
pub enum GetEncodedSursaError {
    #[error("Error: couldn't find '.val(Editor.getValue())' in the response of the problem!\nMaybe the user wasn't logged in or the api of pbinfo changed!")]
    NotFoundEditorGetValue,
}

/// Returns the encoded "sursa" field
///
/// Pbinfo changed their api so that you need an encoded field to be
/// passed to the backend, the field is encoded in the html of
/// the problem page, this function finds it and returns it.
async fn get_encoded_sursa(
    problem_id: &str,
    logged_in_client: &reqwest::Client,
    logged_in_headers: reqwest::header::HeaderMap,
) -> Result<String, Box<dyn std::error::Error>> {
    let request = logged_in_client
        .request(
            reqwest::Method::GET,
            format!("https://www.pbinfo.ro/probleme/{problem_id}"),
        )
        .headers(logged_in_headers);

    let response = request.send().await?;

    let body = response.text().await?;

    // we are looking for the token in a string that looks something
    // like this:
    // {page html}
    // $("#eedd451d5e1eb7dfd9c6e3a0e918f02cc2a87d03").val(Editor.getValue());
    // {continuation page html}
    // and we are extracting "eedd451d5e1eb7dfd9c6e3a0e918f02cc2a87d03"
    let marker = ".val(Editor.getValue())";
    let before = body
        .split(marker)
        .next()
        .ok_or_else(|| GetEncodedSursaError::NotFoundEditorGetValue)?;

    let encoded_sursa_rev: String = before
        .chars()
        .rev()
        .skip_while(|&c| c != '\'')
        .skip(1)
        .take_while(|&c| c != '#')
        .collect();
    let encoded_sursa = encoded_sursa_rev.chars().rev().collect::<String>();

    Ok(encoded_sursa)
}

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

    let encoded_sursa = get_encoded_sursa(problem_id, &client, headers.clone()).await?;

    let form = reqwest::multipart::Form::new()
        .text("limbaj_de_programare", "cpp")
        .text("sursa", "")
        .text(encoded_sursa, source.to_string())
        .text("id", problem_id.to_string());

    let request = client
        .request(
            reqwest::Method::POST,
            "https://www.pbinfo.ro/ajx-module/php-solutie-incarcare.php",
        )
        .headers(headers)
        .multipart(form);

    let response = request.send().await?;

    if response.status() != StatusCode::OK {
        return Err(UploadError::BadStatusCodeError {
            status_code: response.status().as_str().to_string(),
        }
        .into());
    }

    let body = response.text().await?;

    Ok(body)
}

/// Returns the id of a response
///
/// # Arguments
///
/// * `response` - the response to parse
fn get_response_id(response: String) -> Result<String, ResponseIdError> {
    let table: serde_json::Value =
        serde_json::from_str(&response).map_err(|err| ResponseIdError::ParseError {
            json: response.clone(),
            err: err.to_string(),
        })?;
    if table["stare"] != "success" {
        if table["raspuns"] == "Lipsa autentificare" {
            return Err(ResponseIdError::NotLoggedInError);
        }
        if table["raspuns"] == "Așteaptă evaluarea surselor deja trimise" {
            return Err(ResponseIdError::AlreadyEvaluationError);
        }
        if table["raspuns"]
            == "Mai așteaptă! Ai trimis prea multe surse într-un interval scurt de timp."
        {
            return Err(ResponseIdError::CooldownError);
        }
        return Err(ResponseIdError::UnknownUploadError { response: response });
    }
    Ok(table["id_solutie"].to_string())
}

/// Uploads a source and returns a **solution** id
///
/// # Arguments
///
/// * `problem_id` - the id of the problem
/// * `source` - the source to be uploaded for evaluation
/// * `ssid` - ssid of the user (basically login session )
pub async fn upload(problem_id: &str, source: &str, ssid: &str) -> Result<String, UploadError> {
    let response = upload_helper(problem_id, source, ssid)
        .await
        .map_err(|err| UploadError::UploadError {
            err: err.to_string(),
        })?;

    let response_id = get_response_id(response).map_err(|err| match err {
        ResponseIdError::NotLoggedInError => UploadError::NotLoggedInError,
        ResponseIdError::CooldownError => UploadError::CooldownError,
        err => UploadError::ResponseParseError {
            err: err.to_string(),
        },
    })?;
    info!("Upload succefull!");
    Ok(response_id)
}
