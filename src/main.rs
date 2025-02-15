use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;

mod openai;

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
    
    /// Ask a question about the codebase
    #[command(trailing_var_arg = true)]
    Ask {
        /// The question to ask
        #[arg(required = true)]
        prompt: Vec<String>,
    },

    /// Get architectural suggestions for the codebase
    #[command(trailing_var_arg = true)]
    Architect {
        /// The architectural prompt
        #[arg(required = true)]
        prompt: Vec<String>,
    },

    /// Generate or modify code based on a prompt
    #[command(trailing_var_arg = true)]
    Code {
        /// The code generation prompt
        #[arg(required = true)]
        prompt: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment");

    let cli = Cli::parse();

    match cli.command {
        Commands::Watch => {
            println!("Watching for file changes...");
        }
        Commands::Ask { prompt } => {
            let prompt = prompt.join(" ");
            println!("Answering question: {}", prompt);
        }
        Commands::Architect { prompt } => {
            let prompt = prompt.join(" ");
            println!("Providing architectural advice for: {}", prompt);
        }
        Commands::Code { prompt } => {
            let prompt = prompt.join(" ");
            println!("Generating/modifying code for: {}", prompt);
        }
    }
    Ok(())
}
