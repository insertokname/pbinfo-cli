use colored::Colorize;
use log::info;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Error: Json parse failed!")]
    JsonInit,
    #[error("Error: Pbinfo provided an invalid tests json somehow.")]
    InvalidTests,
    #[error("Error: Pbinfo provided an empty test json.")]
    NoTests,
    #[error("Error: Pbinfois still executing the source, will retry.")]
    StillExecuting,
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
    Some(input.strip_prefix("\"")?.strip_suffix("\"")?.to_string())
}

/// Prints out pretty version of score response, if the score is still executing returns StillExecutin error
///
/// # Arguments
///
/// `score` - the score to parse
pub fn parse_score(score: &str) -> Result<(), Box<dyn std::error::Error>> {
    // println!(score);
    let table: serde_json::Value = match serde_json::from_str(score) {
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
        info!(
            "{}{}",
            "\nCompilation Message:\n".bold().underline().red(),
            eval_msg.red()
        );
    }

    let test_array = match teste.as_array() {
        Some(val) => val,
        None => return Err(Box::new(ParseError::NoTests)),
    };
    for i in test_array {
        let cur_pct: i64 = rm_quotes(&i["detalii"]["scor"]).parse().unwrap();
        let max_pct: i64 = rm_quotes(&i["detalii"]["scor_maxim"]).parse()?;

        let is_exemplu: i64 = i["detalii"]["exemplu"].as_i64().unwrap();

        let out = format!(
            "{}: punctaj: {cur_pct}/{max_pct} {} memorie: {}{}",
            rm_quotes(&i["eticheta"]),
            try_remove_sorrounding_quotes(i["detalii"]["mesaj"].to_string())
                .unwrap_or("".to_string())
                .trim_end(),
            rm_quotes(&i["detalii"]["memorie"]),
            if is_exemplu == 1 { "  (exemplu)" } else { "" },
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

#[derive(Debug)]
pub enum ResponseType {
    UnknownUploadError,
    ParseError,
    LipsaAuth,
}

/// Returns the id of a response
///
/// # Arguments
///
/// * `response` - the response to parse
pub fn parse_response(response: &String) -> Result<String, ResponseType> {
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
