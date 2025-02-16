# Luckyshot

A powerful CLI tool that enhances code understanding by using RAG (Retrieval Augmented Generation) to find and analyze relevant files in your codebase.

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
luckyshot scan "*.rs"

# Scan with chunking enabled
luckyshot scan --chunk-size 1000 --overlap-size 100 "src/**/*.rs"

# Include file metadata in embeddings
luckyshot scan --embed-metadata "*.{rs,md}"
```

The scan command:
1. Finds files matching your pattern (respecting .gitignore)
2. Generates embeddings using OpenAI's API
3. Saves results to `.luckyshot.file.vectors.v1`

### Finding Relevant Files

To find files related to a topic or question:

```bash
# Using command line arguments
luckyshot suggest-files "how does the scanning work?"

# Using piped input
echo "how does error handling work?" | luckyshot suggest-files
```

This will:
1. Convert your query into an embedding
2. Use BM25-style ranking to find similar files
3. Display relevant files with similarity scores

### Expanding Context

To expand a query with additional context:

```bash
luckyshot expand "describe the implementation" --system-prompt "You are a helpful assistant"
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
