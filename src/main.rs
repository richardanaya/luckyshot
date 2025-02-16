use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::io::Read;

mod bm25_embedder;
mod bm25_ranker;
mod files;
mod metadata;
mod openai;
mod scan;
mod search;
mod similarity;
mod tokenize_code;

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
        #[arg(short, long, required = true)]
        pattern: String,

        /// Size of chunks to split files into (0 for no chunking)
        #[arg(long, default_value = "0")]
        chunk_size: usize,

        /// Size of overlap between chunks (0 for no overlap)
        #[arg(long, default_value = "0")]
        chunk_overlap: usize,

        /// Include file metadata in embeddings
        #[arg(long, default_value = "false")]
        embed_metadata: bool,
    },

    /// Suggest relevant files based on a query
    SuggestFiles {
        /// The query to find relevant files (optional if using stdin)
        #[arg(short, long, required = false)]
        prompt: Option<String>,

        /// Only return results with similarity >= filter-similarity (0.0 to 1.0)
        #[arg(short, long, default_value = "0.0")]
        filter_similarity: f32,

        /// Show detailed information including similarity scores and chunk details
        #[arg(long, default_value = "false")]
        verbose: bool,

        /// Show the actual contents of matched files/chunks
        #[arg(long, default_value = "false")]
        file_contents: bool,

        /// Limit the number of results (0 for unlimited)
        #[arg(short, long, default_value = "0")]
        count: usize,
    },

    /// Expand a prompt using a system prompt
    Expand {
        /// System prompt for expanding the question
        #[arg(short, long, required = true)]
        system_prompt: String,

        /// The prompt to expand (optional if using stdin)
        #[arg(long, required = false)]
        prompt: Option<String>,
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
            chunk_overlap,
            embed_metadata,
        } => {
            if chunk_size > 0 && chunk_overlap >= chunk_size {
                eprintln!("Error: chunk-overlap must be less than chunk-size");
                std::process::exit(1);
            }
            scan::scan_files(
                &pattern,
                &api_key,
                chunk_size,
                chunk_overlap,
                embed_metadata,
            )
            .await?;
        }
        Commands::SuggestFiles {
            prompt,
            filter_similarity,
            verbose,
            file_contents,
            count,
        } => {
            if !(0.0..=1.0).contains(&filter_similarity) {
                eprintln!("Error: filter-similarity must be between 0.0 and 1.0");
                std::process::exit(1);
            }
            let prompt_text = match prompt {
                None => {
                    // Only try to read from stdin if it's not a terminal
                    if atty::isnt(atty::Stream::Stdin) {
                        let mut buffer = String::new();
                        std::io::stdin().read_to_string(&mut buffer)?;
                        buffer.trim().to_string()
                    } else {
                        String::new()
                    }
                }
                Some(p) => p,
            };

            if prompt_text.is_empty() {
                eprintln!("Error: No prompt given");
                std::process::exit(1);
            }

            if let Err(e) = search::find_related_files(
                &prompt_text,
                &api_key,
                filter_similarity,
                verbose,
                file_contents,
                count,
            )
            .await
            {
                eprintln!("Error finding related files: {}", e);
            }
        }
        Commands::Expand {
            prompt,
            system_prompt,
        } => {
            let prompt_text = match prompt {
                None => {
                    // Only try to read from stdin if it's not a terminal
                    if atty::isnt(atty::Stream::Stdin) {
                        let mut buffer = String::new();
                        std::io::stdin().read_line(&mut buffer)?;
                        buffer
                    } else {
                        String::new()
                    }
                }
                Some(p) => p,
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
