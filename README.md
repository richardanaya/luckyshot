
<p align="center">
  <img src="https://github.com/user-attachments/assets/b58cd03b-1cd8-4d97-b0e1-5498c83df2a3" alt="Description" width="300">
</p>

A powerful CLI tool that enhances code understanding by using RAG (Retrieval Augmented Generation) to find relevant files in your codebase.

## Features

- File scanning with customizable chunk sizes and overlap
- Semantic search using OpenAI embeddings
- Support for piped input and file suggestions
- Intelligent context expansion

## Installation

```bash
cargo install luckyshot
```

## Usage

### Scanning Files

Generate embeddings for your codebase using the `scan` command:

```bash
# Basic scan of all Rust files
luckyshot scan "**/*.rs"

# Basic scan of all Rust and Markdown files
luckyshot scan "**/*{.rs,.md}"

# Scan with chunking enabled
luckyshot scan --chunk-size 1000 --chunk-overlap 100 "src/**/*.rs"

# Include file metadata in embeddings
luckyshot scan --embed-metadata "*.{rs,md}"

# Scan with all options
luckyshot scan --chunk-size 1000 --chunk-overlap 100 --embed-metadata "**/*.rs"
```

The scan command:
1. Finds files matching your pattern (respecting .gitignore)
2. Generates embeddings using OpenAI's API
3. Saves results to `.luckyshot.file.vectors.v1`

### Finding Relevant Files

To find files related to a topic or question:

```bash
# Basic file suggestion
luckyshot suggest-files --prompt "how does the scanning work?"

# Using piped input
echo "how does error handling work?" | luckyshot suggest-files

# Filter results by similarity score (matches >= specified value, range 0.0 to 1.0)
luckyshot suggest-files --prompt "error handling" --filter-similarity 0.5

# Show detailed information including similarity scores
luckyshot suggest-files --prompt "file scanning" --verbose

# Show file contents of matches
luckyshot suggest-files --prompt "metadata handling" --file-contents

# Limit number of results
luckyshot suggest-files --prompt "openai" --count 5

# Combine options
luckyshot suggest-files --prompt "embedding" --verbose --file-contents --filter-similarity 0.7 --count 3

# Chain commands Unix-style
echo "what openai url am I using" | \
  luckyshot expand "you are a rust expert who describes their \
     question and the files you are looking for" | \
  luckyshot suggest-files --verbose
```

This will:
1. Expand your prompt into something with more features to match similarity by
2. Convert your query into an embedding
3. Use BM25-style ranking to find similar files
4. Display relevant files with similarity scores

### Expanding Context

To expand a query with additional context:

```bash
luckyshot expand --system-prompt "You are a helpful assistant" --prompt "describe the implementation"
```

## Environment Setup

You'll need an OpenAI API key. Either:

```bash
export OPENAI_API_KEY="your-api-key"
```

Or create a `.env` file:
```
OPENAI_API_KEY=your-api-key
```

## License

MIT
