# Luckyshot

A powerful CLI tool that enhances AI coding assistants by intelligently selecting relevant context from your codebase using RAG (Retrieval Augmented Generation).

## The Problem

When using AI coding assistants like GitHub Copilot, Aider, or Continue.dev, one of the biggest challenges is selecting which files to include as context. Including too many files overwhelms the AI with irrelevant information, while missing crucial files leads to incomplete understanding and poor suggestions.

Luckyshot solves this by using AI embeddings to automatically find the most relevant files in your codebase for any given query or task. Instead of manually selecting files or using simple text search, Luckyshot uses semantic search to understand the meaning and relationships between your code files.

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
luckyshot scan "src/**/*{.rs,.md}"  # Scan all files in src directory recursively
```

This will:
1. Find all files matching the pattern (respecting your .gitignore)
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
