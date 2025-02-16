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

        /// Only return results with similarity >= filter-similarity (0.0 to 1.0)
        #[arg(long, default_value = "0.0")]
        filter_similarity: f32,

        /// Show detailed information including similarity scores and chunk details
        #[arg(long, default_value = "false")]
        verbose: bool,

        /// Show the actual contents of matched files/chunks
        #[arg(long, default_value = "false")]
        file_contents: bool,

        /// Limit the number of results (0 for unlimited)
        #[arg(long, default_value = "0")]
        count: usize,
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
        Commands::SuggestFiles { prompt, filter_similarity, verbose, file_contents, count } => {
            if !(0.0..=1.0).contains(&filter_similarity) {
                eprintln!("Error: filter-similarity must be between 0.0 and 1.0");
                std::process::exit(1);
            }
            let prompt_text = if prompt.is_empty() {
                // Only try to read from stdin if it's not a terminal
                if atty::isnt(atty::Stream::Stdin) {
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer)?;
                    buffer.trim().to_string()
                } else {
                    String::new()
                }
            } else {
                prompt.join(" ").trim().to_string()
            };

            if prompt_text.is_empty() {
                eprintln!("Error: No prompt given");
                std::process::exit(1);
            }

            match openai::get_embedding(&prompt_text, &api_key).await {
                Ok(embedding) => {
                    let _related_files = openai::find_related_files(embedding, filter_similarity, verbose, file_contents, count).await;
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
                // Only try to read from stdin if it's not a terminal
                if atty::isnt(atty::Stream::Stdin) {
                    let mut buffer = String::new();
                    std::io::stdin().read_line(&mut buffer)?;
                    buffer
                } else {
                    String::new()
                }
            } else {
                prompt.join(" ")
            };

            if prompt_text.is_empty() {
                eprintln!("Error: No prompt given");
                std::process::exit(1);
            }

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
