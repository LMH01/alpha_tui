use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "LMH01", version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, long_help = "Specify the input file that contains the program", required = true)]
    pub input: String,
}