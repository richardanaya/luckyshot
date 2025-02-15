fn main() {
    println!("Hello, world!");
}
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch for file changes
    Watch,
    
    /// Execute a one-shot generation with the given prompt
    #[command(trailing_var_arg = true)]
    Make {
        /// The prompt to process
        #[arg(required = true)]
        prompt: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch => {
            println!("Watching for file changes...");
        }
        Commands::Make { prompt } => {
            let prompt = prompt.join(" ");
            println!("Processing prompt: {}", prompt);
        }
    }
}
