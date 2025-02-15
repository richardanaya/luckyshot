use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use glob_match::glob_match;
use std::fs;
use std::collections::HashMap;

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
        Commands::Scan { pattern } => {
            println!("Scanning for files matching pattern: {}", pattern);
            let mut file_embeddings: HashMap<String, Vec<f32>> = HashMap::new();
            
            // Recursively get all files in current directory
            for entry in std::fs::read_dir(".").expect("Failed to read directory") {
                if let Ok(entry) = entry {
                    if let Ok(path) = entry.path().canonicalize() {
                        let path_str = path.to_string_lossy().to_string();
                        // Skip the vectors file itself
                        if path_str.ends_with(".luckyshot.file.vectors.v1") {
                            continue;
                        }
                        // Only process files that match the pattern
                        if !glob_match(&pattern, &path_str) {
                            continue;
                        }
                        println!("Processing: {}", path_str);
                        
                        match fs::read_to_string(&path) {
                            Ok(contents) => {
                                match openai::get_embedding(&contents, &api_key).await {
                                    Ok(embedding) => {
                                        println!("Got embedding for {} (length {})", path_str, embedding.len());
                                        file_embeddings.insert(path_str, embedding);
                                    }
                                    Err(e) => eprintln!("Error getting embedding for {}: {}", path_str, e),
                                }
                            }
                            Err(e) => eprintln!("Error reading file {}: {}", path_str, e),
                        }
                    }
                    Err(e) => println!("Error with path: {}", e),
                }
            }
            
            // Save embeddings to file
            match serde_json::to_string_pretty(&file_embeddings) {
                Ok(json) => {
                    if let Err(e) = fs::write(".luckyshot.file.vectors.v1", json) {
                        eprintln!("Error writing vectors file: {}", e);
                    } else {
                        println!("Successfully saved vectors for {} files", file_embeddings.len());
                    }
                }
                Err(e) => eprintln!("Error serializing vectors: {}", e),
            }
        }
        Commands::Ask { prompt } => {
            let prompt = prompt.join(" ");
            println!("Answering question: {}", prompt);
            
            match openai::get_embedding(&prompt, &api_key).await {
                Ok(embedding) => {
                    println!("Got embedding vector (length {})", embedding.len());
                    let related_files = openai::find_related_files(embedding).await;
                    println!("Related files: {:?}", related_files);
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
        Commands::Architect { prompt } => {
            let prompt = prompt.join(" ");
            println!("Providing architectural advice for: {}", prompt);
            
            match openai::get_embedding(&prompt, &api_key).await {
                Ok(embedding) => {
                    println!("Embedding vector (length {}): {:?}", embedding.len(), embedding);
                    let related_files = openai::find_related_files(embedding).await;
                    println!("Related files: {:?}", related_files);
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
        Commands::Code { prompt } => {
            let prompt = prompt.join(" ");
            println!("Generating/modifying code for: {}", prompt);
            
            match openai::get_embedding(&prompt, &api_key).await {
                Ok(embedding) => {
                    println!("Embedding vector (length {}): {:?}", embedding.len(), embedding);
                    let related_files = openai::find_related_files(embedding).await;
                    println!("Related files: {:?}", related_files);
                }
                Err(e) => {
                    eprintln!("Error getting embedding: {}", e);
                }
            }
        }
    }
    Ok(())
}
