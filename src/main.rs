use clap::Parser;

use colored::Colorize;

mod config;

#[derive(Debug)]
enum ResponseType {
    UnknownUploadError,
    ParseError,
    LipsaAuth,
}

#[derive(thiserror::Error, Debug)]
enum CookieError {
    #[error("Error: Pbinfo didn't set the ssid cookie!\nLogin failed!")]
    NoCookie,
    #[error("Error: Got an improperly formated cookie!")]
    BadCookie,
}

#[derive(thiserror::Error, Debug)]
enum UploadError {
    #[error("Error: The user is not logged in!\nThe password or the email may be incorect!")]
    NotLoggedIn,
}

#[derive(thiserror::Error, Debug, PartialEq)]
enum ParseError {
    #[error("Error: Json parse failed!")]
    JsonInit,
    #[error("Error: Pbinfo provided an invalid tests json somehow.")]
    InvalidTests,
    #[error("Error: Pbinfo provided an empty test json.")]
    NoTests,
    #[error("Error: Pbinfois still executing the source, will retry.")]
    StillExecuting,
}

async fn upload_solution(
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

fn try_get_ssid(response: &reqwest::Response)->Result<String,Box<dyn std::error::Error>>{
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
        None => return Err(Box::new(CookieError::BadCookie)),}

}

async fn login(
    email: &str,
    password: &str,
    form_token: &mut String,
    ssid: &mut String,
) -> Result<(), Box<dyn std::error::Error>> {
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
    
    if let Ok(new_ssid) = try_get_ssid(&response){
        *ssid=new_ssid;
        return Ok(());
    }
    let text = response.text().await?;
    let table: serde_json::Value = serde_json::from_str(&text)?;
    let new_form_token = table["form_token"].to_string();


    dbg!(&table);
    println!("first{}",*form_token);
    *form_token = new_form_token[1..new_form_token.len() - 1].to_string();
    println!("first{}",*form_token);

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


    *ssid = try_get_ssid(&response).unwrap();

    Ok(())
}

fn parse_response(response: &String) -> Result<String, ResponseType> {
    let table: serde_json::Value = match serde_json::from_str(&response) {
        Ok(val) => val,
        Err(_) => return Err(ResponseType::ParseError),
    };
    if table["stare"] != "success" {
        if table["raspuns"] == "Lipsa autentificare" {
            return Err(ResponseType::LipsaAuth);
        }
        return Err(ResponseType::UnknownUploadError);
    }
    Ok(table["id_solutie"].to_string())
}

async fn try_upload(
    id: &str,
    source: &str,
    ssid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = match upload_solution(id, source, ssid).await {
        Ok(val) => val,
        Err(_) => {
            println!("ERROR:\n\tCould not connect to pbinfo!\n\tCheck network connection and that pbinfo dns is up.");
            std::process::exit(1);
        }
    };

    match parse_response(&response) {
        Ok(val) => {
            println!("Upload succefull!");
            return Ok(val);
        }
        Err(err) => match err {
            ResponseType::ParseError => {
                println!("ERROR:\n\tDidn't get a valid response: {}", response);
                std::process::exit(1);
            }
            ResponseType::UnknownUploadError => {
                println!("ERROR:\n\tINVALID RESPONSE:{}", response);
            }
            ResponseType::LipsaAuth => {
                println!("The user is not logged in!");
                return Err(Box::new(UploadError::NotLoggedIn));
            }
        },
    }

    Ok("ID".to_string())
}

async fn get_score(
    sol_id: &str,
    ssid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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

fn rm_quotes(input: &serde_json::Value) -> String {
    input
        .as_str()
        .unwrap_or("0")
        .chars()
        .filter(|c| c != &'\"')
        .collect::<String>()
}

fn try_remove_sorrounding_quotes(input: String) -> Option<String> {
    Some(
        input
            .strip_prefix("\"")?
            .strip_suffix("\"")?
            .to_string(),
    )
}

fn parse_score(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    // println!(input);
    let table: serde_json::Value = match serde_json::from_str(input) {
        Ok(val) => val,
        Err(_) => return Err(Box::new(ParseError::JsonInit)),
    };

    if table["status_sursa"] == "executing" || table["status_sursa"] == "pending" {
        return Err(Box::new(ParseError::StillExecuting));
    }

    let teste: serde_json::Value = match serde_json::from_value(table["teste"].clone()) {
        Ok(val) => val,
        Err(_) => return Err(Box::new(ParseError::InvalidTests)),
    };

    let eval_msg = try_remove_sorrounding_quotes(
        table["mesaj_compilare_brut"]
            .to_string()
            .replace("\\n", "\n")
            .replace("\\r", "\r"),
    )
    .unwrap_or("".to_string());

    if eval_msg != "" {
        println!(
            "{}{}",
            "\nCompilation Message:\n"
                .bold()
                .underline()
                .red(),
            eval_msg.red()
        );
    }

    let test_array = match teste.as_array() {
        Some(val) => val,
        None => return Err(Box::new(ParseError::NoTests)),
    };
    for i in test_array {
        let cur_pct: i64 = rm_quotes(&i["detalii"]["scor"])
            .parse()
            .unwrap();
        let max_pct: i64 = rm_quotes(&i["detalii"]["scor_maxim"]).parse()?;

        let is_exemplu: i64 = i["detalii"]["exemplu"]
            .as_i64()
            .unwrap();

        let out = format!(
            "{}: punctaj: {cur_pct}/{max_pct} {} memorie: {}{}",
            rm_quotes(&i["eticheta"]),
            try_remove_sorrounding_quotes(i["detalii"]["mesaj"].to_string())
                .unwrap_or("".to_string())
                .trim_end(),
            rm_quotes(&i["detalii"]["memorie"]),
            if is_exemplu == 1 {
                "  (exemplu)"
            } else {
                ""
            },
        );
        if max_pct <= cur_pct {
            println!("{}", out.green().bold());
        } else if cur_pct == 0 {
            println!("{}", out.red().bold());
        } else {
            println!(
                "{}",
                out.custom_color(colored::CustomColor::new(248, 213, 104))
                    .bold()
            );
        }
        // println!();
        // print!("{i}\n\n");
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the source file
    #[arg(short, long, default_value_t={"main.cpp".to_string()})]
    filename: String,

    /// If set asks for a new email when run
    #[arg(
        long,
    )]
    email:bool,

    /// If set asks for a new password when run
    #[arg(
        long,
    )]
    reset_password:bool,

    #[arg(
        short,
        long
    )]
    id_problema:String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut config = config::get_config();

    // println!("{:#?}", config);

    if config.email == "" || args.email{
        println!("Enter email:");
        config.email.clear();
        std::io::stdin()
            .read_line(&mut config.email)
            .expect("invalid email!");

        config.email= config.email.trim().to_string();
        config::save_config(&config);

    }

    if config.password == "" || args.reset_password{
        println!("Enter password:");
        config.password.clear();
        std::io::stdin()
            .read_line(&mut config.password)
            .expect("invalid password!");
        config.password = config.password.trim().to_string();
        config::save_config(&config);
    };

    let source = std::fs::read_to_string(args.filename).expect("Could not read source file!\n");
    if source.is_empty(){
        println!("Given source file was empty!");
        std::process::exit(1);
    }

    // settings.get("ssid").expect("didn't find the ssid in config");
    // settings.get("form_token").expect("didn't find the form_token in config");
    // let mut ssid: String = "mrlkokpsm6p43r8h4p4u784ujv".to_string();
    // let mut form_token: String = "mrlkokpsm6p43r8h4p4u784ujv".to_string();

    // let file = std::include_str!("other-score.json");

    println!("Uploading solution...");
    let solution_id = match try_upload(&args.id_problema, &source, &config.ssid).await {
        Ok(val) => val,
        Err(_) => {
            println!("Attempting to login!");
            match login(
                &config.email,
                &config.password,
                &mut config.form_token,
                &mut config.ssid,
            )
            .await
            {
                Ok(val) => {
                    config::save_config(&config);
                    
                    val
                }
                Err(err) => {
                    println!("COULD NOT LOGIN: \n{err}");
                    std::process::exit(1);
                }
            };
            println!("Login succesfull!");
            match try_upload(&args.id_problema, &source, &config.ssid).await {
                Ok(val) => val,
                Err(_) => {
                    println!("The password or the email may be incorect, please double check!");
                    std::process::exit(1);
                }
            }
        }
    };

    // println!("Upload success! solution id:{solution_id}");

    // tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    // let answers = get_score(&solution_id, &config.ssid)
    //     .await
    //     .unwrap();

    // println!("{answers}");

    println!("Program is being evaluated!");
    while let Err(err) = parse_score(
        &get_score(&solution_id, &config.ssid)
            .await
            .unwrap(),
    ) {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        if let Some(down_err) = err.downcast_ref::<ParseError>() {
            if *down_err == ParseError::StillExecuting {
                println!("Program is still being evaluated...!");
                continue;
            }
        }
        break;
    }

}
