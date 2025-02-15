use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;

mod files;
mod openai;
mod scan;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan files matching a glob pattern
    Scan {
        /// The glob pattern to match files
        #[arg(required = true)]
        pattern: String,
    },

    /// Ask a question about the codebase
    #[command(trailing_var_arg = true)]
    Ask {
        /// The question to ask
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
        Commands::Scan { pattern } => {
            scan::scan_files(&pattern, &api_key).await?;
        }
        Commands::Ask { prompt } => {
            let prompt = prompt.join(" ");
            println!("Answering question: {}", prompt);

            match openai::get_embedding(&prompt, &api_key).await {
                Ok(embedding) => {
                    println!("Got embedding vector (length {})", embedding.len());
                    let related_files = openai::find_related_files(embedding).await;
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
    }
    Ok(())
}
