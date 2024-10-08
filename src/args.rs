use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the source file
    #[arg(short, long, default_value_t={"main.cpp".to_string()})]
    pub filename: String,

    /// If set asks for a new email when run
    #[arg(long)]
    pub reset_email: bool,

    /// If set asks for a new password when run
    #[arg(long)]
    pub reset_password: bool,

    #[arg(short, long)]
    pub problem_id: String,
}