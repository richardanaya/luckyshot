use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::io::Read;

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

    /// Suggest relevant files based on a query
    #[command(trailing_var_arg = true)]
    SuggestFiles {
        /// The query to find relevant files (optional if using stdin)
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
    },

    /// Expand a prompt using a system prompt
    Expand {
        /// System prompt for expanding the question
        #[arg(required = true)]
        system_prompt: String,

        /// The prompt to expand (optional if using stdin)
        #[arg(trailing_var_arg = true)]
        prompt: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found in environment");

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            pattern,
            chunk_size,
            overlap_size,
            embed_metadata,
        } => {
            if chunk_size > 0 && overlap_size >= chunk_size {
                eprintln!("Error: overlap_size must be less than chunk_size");
                std::process::exit(1);
            }
            scan::scan_files(&pattern, &api_key, chunk_size, overlap_size, embed_metadata).await?;
        }
        Commands::SuggestFiles { prompt } => {
            let prompt_text = if prompt.is_empty() {
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            } else {
                prompt.join(" ")
            };

            if prompt_text.is_empty() {
                eprintln!("Error: No input provided via arguments or stdin");
                std::process::exit(1);
            }

            match openai::get_embedding(&prompt_text, &api_key).await {
                Ok(embedding) => {
                    let _related_files = openai::find_related_files(embedding).await;
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
        Commands::Expand {
            prompt,
            system_prompt,
        } => {
            let prompt_text = if prompt.is_empty() {
                // Read from stdin if no prompt arguments provided
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                buffer
            } else {
                prompt.join(" ")
            };

            if !prompt_text.trim().is_empty() {
                match openai::get_openai_chat_completion(&prompt_text, &system_prompt, &api_key)
                    .await
                {
                    Ok(expanded) => println!("{}", expanded),
                    Err(e) => eprintln!("Error expanding prompt: {}", e),
                }
            } else {
                eprintln!("Error: No prompt provided via arguments or stdin");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
