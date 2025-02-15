# Luckyshot

A CLI tool for intelligent code generation using RAG (Retrieval Augmented Generation) and file watching.

## Features

- File scanning and embedding generation
- Semantic search across codebase
- Intelligent code generation using context-aware prompts
- File change watching (coming soon)

## Installation

```bash
cargo install luckyshot
```

## Usage

### Scanning Files

To generate embeddings for your codebase, use the `scan` command with a glob pattern:

```bash
luckyshot scan "*.rs"  # Scan all Rust files
luckyshot scan "src/**/*"  # Scan all files in src directory recursively
```

This will:
1. Find all files matching the pattern
2. Generate embeddings using OpenAI's API
3. Save the embeddings to `.luckyshot.file.vectors.v1`

### Asking Questions

To ask questions about your codebase:

```bash
luckyshot ask "how does the file scanning work?"
```

This will:
1. Convert your question into an embedding
2. Find the most semantically similar files in your codebase
3. Use those files as context to provide a relevant answer

## Environment Setup

You'll need an OpenAI API key to use Luckyshot. Set it in your environment:

```bash
export OPENAI_API_KEY="your-api-key"
```

Or create a `.env` file in your project root:

```
OPENAI_API_KEY=your-api-key
```

## License

MIT
