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

        /// Size of chunks to split files into (0 for no chunking)
        #[arg(long, default_value = "0")]
        chunk_size: usize,

        /// Size of overlap between chunks (0 for no overlap)
        #[arg(long, default_value = "0")]
        overlap_size: usize,

        /// Include file metadata in embeddings
        #[arg(long, default_value = "false")]
        embed_metadata: bool,
    },

    /// Ask a question about the codebase
    #[command(trailing_var_arg = true)]
    Ask {
        /// The question to ask
        #[arg(required = true)]
        prompt: Vec<String>,

        /// Optional system prompt for expanding the question
        #[arg(long)]
        prompt_expansion: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment");

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { pattern, chunk_size, overlap_size, embed_metadata } => {
            if chunk_size > 0 && overlap_size >= chunk_size {
                eprintln!("Error: overlap_size must be less than chunk_size");
                std::process::exit(1);
            }
            scan::scan_files(&pattern, &api_key, chunk_size, overlap_size, embed_metadata).await?;
        }
        Commands::Ask { prompt, prompt_expansion } => {
            let prompt = prompt.join(" ");
            println!("Answering question: {}", prompt);

            // If prompt_expansion is provided, first expand the prompt
            let expanded_prompt = if let Some(system_prompt) = prompt_expansion {
                match openai::get_openai_chat_completion(&prompt, &system_prompt, &api_key).await {
                    Ok(expanded) => expanded,
                    Err(e) => {
                        eprintln!("Error expanding prompt: {}", e);
                        prompt
                    }
                }
            } else {
                prompt
            };

            match openai::get_embedding(&expanded_prompt, &api_key).await {
                Ok(embedding) => {
                    let _related_files = openai::find_related_files(embedding).await;
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
    }
    Ok(())
}
