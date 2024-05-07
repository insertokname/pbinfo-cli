#[derive(thiserror::Error, Debug)]
enum CookieError {
    #[error("Error: Pbinfo didn't set the ssid cookie!\nLogin failed!")]
    NoCookie,
    #[error("Error: Got an improperly formated cookie!")]
    BadCookie,
}
fn try_get_ssid(response: &reqwest::Response) -> Result<String, Box<dyn std::error::Error>> {
    let new_ssid_header: &str = match response.headers().get("set-cookie") {
        Some(val) => val,
        None => return Err(Box::new(CookieError::NoCookie)),
    }
    .to_str()?;

    let new_ssid_cookie = match new_ssid_header.split(";").next() {
        Some(val) => val,
        None => return Err(Box::new(CookieError::BadCookie)),
    };

    match new_ssid_cookie.split("=").nth(1) {
        Some(val) => Ok(val.to_string()),
        None => return Err(Box::new(CookieError::BadCookie)),
    }
}

enum LoginHelperStatus {
    RenewedEverything,
    RenewedToken,
}

async fn login_helper(
    email: &str,
    password: &str,
    form_token: &mut String,
    ssid: &mut String,
) -> Result<LoginHelperStatus, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Origin", "https://www.pbinfo.ro".parse()?);
    headers.insert("Referer", "https://www.pbinfo.ro/".parse()?);
    headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let form_data = reqwest::multipart::Form::new()
        .text("user", email.to_string())
        .text("parola", password.to_string())
        .text("form_token", form_token.to_string());

    let request = client
        .request(
            reqwest::Method::POST,
            "https://www.pbinfo.ro/ajx-module/php-login.php",
        )
        .headers(headers)
        .multipart(form_data);

    let response = request.send().await?;

    if let Ok(new_ssid) = try_get_ssid(&response) {
        *ssid = new_ssid;
        return Ok(LoginHelperStatus::RenewedToken);
    }
    let text = response.text().await?;
    let table: serde_json::Value = serde_json::from_str(&text)?;
    let new_form_token = table["form_token"].to_string();

    // dbg!(&table);
    // println!("first{}",*form_token);
    *form_token = new_form_token[1..new_form_token.len() - 1].to_string();
    // println!("first{}",*form_token);

    let mut new_headers = reqwest::header::HeaderMap::new();
    new_headers.insert("Origin", "https://www.pbinfo.ro".parse()?);
    new_headers.insert("Referer", "https://www.pbinfo.ro/".parse()?);
    new_headers.insert("Cookie", format!("SSID={ssid}").parse()?);

    let new_form_data = reqwest::multipart::Form::new()
        .text("user", email.to_string())
        .text("parola", password.to_string())
        .text("form_token", form_token.to_string());

    let new_request = client
        .request(
            reqwest::Method::POST,
            "https://www.pbinfo.ro/ajx-module/php-login.php",
        )
        .headers(new_headers)
        .multipart(new_form_data);

    let response = new_request.send().await?;

    *ssid = try_get_ssid(&response)
        .expect("\nPbinfo did not send back an ssid.\nThe username and password may be incorect!");

    Ok(LoginHelperStatus::RenewedEverything)
}

/// Logs in a user and sets the ssid and form token to the correct values
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
) -> Result<(), Box<dyn std::error::Error>> {
    match login_helper(email, password, form_token, ssid).await? {
        LoginHelperStatus::RenewedToken => {
            login_helper(email, password, form_token, ssid).await?;
            return Ok(());
        }
        LoginHelperStatus::RenewedEverything => {
            return Ok(());
        }
    }
}
