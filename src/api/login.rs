use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Error: didn't get back an ssid cookie!\nLogin failed!\nUsername/Email and password may be incorect! OR Maybe you tried logging in too many times!")]
    NoCookieError,
    #[error("Error: Couldn't parse a header!\nGot error:\n{err}")]
    HeaderParseError { err: String },
    #[error("Error: Couldn't parse the following cookie:\n{cookie}!\nGot error:\n{err}")]
    CookieParseError { cookie: String, err: String },
    #[error("Error: Couldn't send a request to the url: {url}\nGot error:\n{err}")]
    RequestSendError { url: String, err: String },
    #[error("Error: Couldn't build a reqwest client\nGot error:\n{err}")]
    RequestBuildError { err: String },
    #[error("Error: Couldn't parse a response\nGot error:\n{err}")]
    ResponseParseError { err: String },
    #[error("Error: Couldn't parse the following text to a json:\n{json}\nGot error:\n{err}")]
    JsonParseError { json: String, err: String },
}

impl From<InvalidHeaderValue> for LoginError {
    fn from(err: InvalidHeaderValue) -> Self {
        Self::HeaderParseError {
            err: err.to_string(),
        }
    }
}

async fn is_logged_in(ssid: &str) -> Result<bool, LoginError> {
    let client: reqwest::Client =
        reqwest::Client::builder()
            .build()
            .map_err(|err| LoginError::RequestBuildError {
                err: err.to_string(),
            })?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let verify_login_url = "https://www.pbinfo.ro/ajx-module/php-verificare-mesaje-noi.php";
    let response = client
        .request(reqwest::Method::POST, verify_login_url)
        .headers(headers)
        .send()
        .await
        .map_err(|err| LoginError::RequestSendError {
            url: verify_login_url.to_string(),
            err: err.to_string(),
        })?;

    let text = response
        .text()
        .await
        .map_err(|err| LoginError::ResponseParseError {
            err: err.to_string(),
        })?;

    if text.is_empty() {
        return Ok(false);
    }

    let table: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| LoginError::JsonParseError {
            json: text,
            err: err.to_string(),
        })?;

    Ok(table.get("conversatii").is_some())
}

fn try_get_ssid(response: &reqwest::Response) -> Result<String, LoginError> {
    let new_ssid_header = response
        .headers()
        .get("set-cookie")
        .ok_or_else(|| LoginError::NoCookieError)?
        .to_str()
        .map_err(|err| LoginError::HeaderParseError {
            err: format!(
                "Couldn't make a string out of the HeaderValue, got error: {}",
                err.to_string()
            ),
        })?;

    let new_ssid_cookie =
        new_ssid_header
            .split(";")
            .next()
            .ok_or_else(|| LoginError::HeaderParseError {
                err: format!(
                    "Couldn't find anything after the first ';' in the header:\n{new_ssid_header}"
                ),
            })?;

    new_ssid_cookie
        .split("=")
        .nth(1)
        .ok_or_else(|| LoginError::CookieParseError {
            cookie: new_ssid_cookie.to_string(),
            err: "Couldn't find anything after the '=' sign!".to_string(),
        })
        .map(|x| x.to_string())
}

enum LoginHelperStatus {
    RenewedEverything,
    RenewedSsid,
}

async fn login_helper(
    email: &str,
    password: &str,
    form_token: &mut String,
    ssid: &mut String,
) -> Result<LoginHelperStatus, LoginError> {
    let client: reqwest::Client =
        reqwest::Client::builder()
            .build()
            .map_err(|err| LoginError::RequestBuildError {
                err: err.to_string(),
            })?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Origin", "https://www.pbinfo.ro".parse()?);
    headers.insert("Referer", "https://www.pbinfo.ro/".parse()?);
    headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let form_data = reqwest::multipart::Form::new()
        .text("user", email.to_string())
        .text("parola", password.to_string())
        .text("form_token", form_token.to_string());

    let login_url = "https://www.pbinfo.ro/ajx-module/php-login.php";
    let response = client
        .request(reqwest::Method::POST, login_url)
        .headers(headers)
        .multipart(form_data)
        .send()
        .await
        .map_err(|err| LoginError::RequestSendError {
            url: login_url.to_string(),
            err: err.to_string(),
        })?;

    if let Ok(new_ssid) = try_get_ssid(&response) {
        *ssid = new_ssid;
        return Ok(LoginHelperStatus::RenewedSsid);
    }

    let text = response
        .text()
        .await
        .map_err(|err| LoginError::ResponseParseError {
            err: err.to_string(),
        })?;

    let table: serde_json::Value =
        serde_json::from_str(&text).map_err(|err| LoginError::JsonParseError {
            json: text,
            err: err.to_string(),
        })?;
    let new_form_token = table["form_token"].to_string();

    *form_token = new_form_token[1..new_form_token.len() - 1].to_string();

    let mut new_headers = reqwest::header::HeaderMap::new();
    new_headers.insert("Origin", "https://www.pbinfo.ro".parse()?);
    new_headers.insert("Referer", "https://www.pbinfo.ro/".parse()?);
    new_headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let new_form_data = reqwest::multipart::Form::new()
        .text("user", email.to_string())
        .text("parola", password.to_string())
        .text("form_token", form_token.to_string());

    let login_url = "https://www.pbinfo.ro/ajx-module/php-login.php";
    let response = client
        .request(reqwest::Method::POST, login_url)
        .headers(new_headers)
        .multipart(new_form_data)
        .send()
        .await
        .map_err(|err| LoginError::RequestSendError {
            url: login_url.to_string(),
            err: err.to_string(),
        })?;

    *ssid = try_get_ssid(&response)?;

    Ok(LoginHelperStatus::RenewedEverything)
}

/// Makes sure a user is logged in, if not logs in the user with the 
/// provided credentials
///
/// # Arguments
///
/// * `email` - The email of the user to be logged in
/// * `password` - The password of the user to be logged in
/// * `form_token` - The form_token of the user to be logged in
/// * `ssid` - The ssid of the user to be logged in
pub async fn login(
    email: &str,
    password: &str,
    form_token: &mut String,
    ssid: &mut String,
) -> Result<(), LoginError> {
    if let Ok(is_logged_in) = is_logged_in(&ssid).await {
        if is_logged_in {
            log::info!("User already logged in!");
            return Ok(());
        }
    }

    match login_helper(email, password, form_token, ssid).await? {
        LoginHelperStatus::RenewedSsid => {
            login_helper(email, password, form_token, ssid).await?;
            return Ok(());
        }
        LoginHelperStatus::RenewedEverything => {
            return Ok(());
        }
    }
}
