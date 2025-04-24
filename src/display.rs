use colored::Colorize;
use log::info;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Error: Couldn't parse a response json:\n{json}\nError was: {err}")]
    ParseJsonError { json: String, err: String },
    #[error(
        "Error: 'teste' field in response json was not an array!\nInstead got:\n{teste_field}"
    )]
    InvalidTestFormattingError { teste_field: String },
    #[error("Error: A test had an invalid integer value it couldn't be parsed to a i64!\nInvalid field was: {invalid_test}")]
    InvalidScoreFormat { invalid_test: String },
    #[error("Error: Solution is still being evaluated!\nPlease try again in a few seconds!")]
    StillExecutingError,
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

/// Parses and prints out pretty version of score response
///
/// # Arguments
///
/// `table` - the json form of the scores
pub fn display_score(table: Value) -> Result<(), DisplayError> {
    if table["status_sursa"] == "executing" || table["status_sursa"] == "pending" {
        return Err(DisplayError::StillExecutingError);
    }

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

    let test_array =
        table["teste"]
            .as_array()
            .ok_or_else(|| DisplayError::InvalidTestFormattingError {
                teste_field: table["teste"].to_string(),
            })?;

    for i in test_array {
        let cur_pct: i64 = rm_quotes(&i["detalii"]["scor"]).parse().map_err(|_| {
            DisplayError::InvalidScoreFormat {
                invalid_test: i["detalii"]["scor"].to_string(),
            }
        })?;
        let max_pct: i64 = rm_quotes(&i["detalii"]["scor_maxim"])
            .parse()
            .map_err(|_| DisplayError::InvalidScoreFormat {
                invalid_test: i["detalii"]["scor_maxim"].to_string(),
            })?;

        let is_exemplu: i64 = i["detalii"]["exemplu"].as_i64().unwrap();

        let out = format!(
            "{}: punctaj: {cur_pct}/{max_pct} {} memorie: {}{}",
            rm_quotes(&i["eticheta"]),
            try_remove_sorrounding_quotes(i["detalii"]["mesaj"].to_string())
                .unwrap_or(i["detalii"]["mesaj"].to_string())
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
    }

    Ok(())
}
