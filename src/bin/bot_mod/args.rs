use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// If set asks for a new email when run
    #[arg(long, short = 'E')]
    pub reset_email: bool,

    /// If set asks for a new password when run
    #[arg(long, short = 'P')]
    pub reset_password: bool,

    /// If set asks for a new user_id when run
    #[arg(long, short = 'U')]
    pub reset_user_id: bool,

    /// The id of the problem to start at
    #[arg(short = 's', long, default_value_t = 0)]
    pub start_problem_id: u32,

    /// The id of the problem to end at
    #[arg(short = 'e', long, default_value_t = 6000)]
    pub end_problem_id: u32,

    /// The delay (in milliseconds) in between each score read (may help avoid bot detection or throttling from pbinfo.ro but slows down execution)
    #[arg(short = 'D', long, default_value_t = 1000)]
    pub delay: u64,

    /// If not set will just do a dry run that will just find all your solved and unsolved problems
    #[arg(short = 'd', long)]
    pub dry_run: bool,
}
