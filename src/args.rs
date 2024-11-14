use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Configuration file to read.
    #[arg(short, long, default_value = "0.0.0.0:12345")]
    listen: String,
}


impl Args {
    pub fn new() -> Args {
        let args = Args::parse();

        args
    }

    pub fn listen(&self) -> &String {
        &self.listen
    }
}
