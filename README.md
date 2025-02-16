<p align="center">
  <img src="https://github.com/user-attachments/assets/b58cd03b-1cd8-4d97-b0e1-5498c83df2a3" alt="Description" width="300">
</p>

A powerful CLI tool that enhances code understanding and automation by finding the most relevant files in your codebase for AI-assisted programming.

## Why This Tool?

Finding the right files to manipulate with AI is crucial for effective code generation and modification. Traditional approaches like grep or fuzzy finding often miss semantically relevant files that don't contain exact keyword matches.

This tool uses a hybrid approach combining two powerful search techniques:

1. BM25 Ranking: A battle-tested information retrieval algorithm (used by search engines) that excels at keyword matching while accounting for term frequency and document length. It's particularly good at finding files containing specific technical terms or function names.

2. RAG (Retrieval Augmented Generation) with Embedding Distance: Uses OpenAI's embeddings to capture the semantic meaning of both your query and codebase. By measuring vector dot product distances, it can find conceptually related files even when they use different terminology.

The hybrid scoring system combines both approaches:
- BM25 helps catch direct matches and technical terms
- Embedding distance captures semantic relationships and higher-level concepts
- Results are normalized and merged to give you the most relevant files for your task

This dual approach helps ensure you don't miss important context when using AI to modify your codebase.

# Warnings!

This tool is alpha and not thoroughly evalulated with real world tests.

## Features

- File scanning with customizable chunk sizes and overlap
- Semantic search using OpenAI embeddings and BM25 ranking
- Support for piped input and file suggestions
- Intelligent context expansion
- Supports Unix-philosophy piped commands

## Installation

```bash
cargo install luckyshot
```

## Usage

### Scanning Files

Generate embeddings for your codebase using the `scan` command:

```bash
# Basic scan of all Rust files
luckyshot scan -p "**/*.rs"

# Basic scan of all Rust and Markdown files
luckyshot scan -p "**/*{.rs,.md}"

# Scan with chunking enabled
luckyshot scan --chunk-size 1000 --chunk-overlap 100 -p "src/**/*.rs"

# Include file metadata in embeddings
luckyshot scan --embed-metadata "*.{rs,md}"

# Scan with all options
luckyshot scan --chunk-size 1000 --chunk-overlap 100 --embed-metadata -p "**/*.rs"
```

The scan command:
1. Finds files matching your pattern (respecting .gitignore)
2. Generates embeddings using OpenAI's API
3. Saves results to `.luckyshot.file.vectors.v1`

### Finding Relevant Files

To find files related to a topic or question:

```bash
# Basic file suggestion
luckyshot suggest-files -p "how does the scanning work?"

# Using piped input
echo "how does error handling work?" | luckyshot suggest-files

# Filter results by similarity score (matches >= specified value, range 0.0 to 1.0)
luckyshot suggest-files -p "error handling" --filter-similarity 0.5

# Show detailed information including similarity scores
luckyshot suggest-files -p "file scanning" --verbose

# Show file contents of matches
luckyshot suggest-files -p "metadata handling" --file-contents

# Limit number of results
luckyshot suggest-files -p "openai" --count 5

# Combine options
luckyshot suggest-files -p "embedding" --verbose --file-contents --filter-similarity 0.7 --count 3

 # Chain commands Unix-style                                                                                                                                                              
 echo "what openai url am I using" | \                                                                                                                                                    
   luckyshot expand "you are a rust expert who describes their \                                                                                                                          
      question and the files you are looking for" | \                                                                                                                                     
   luckyshot suggest-files --verbose  
```

This will:
1. Convert your query into an embedding
2. Use cross-product ranking to find similar file embedding
3. Display relevant files with similarity scores

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

## Hybrid Algorithm

The tool uses a novel hybrid approach combining BM25 and embedding-based similarity:

1. BM25 Scoring: Produces both positive and negative scores
   - Positive scores indicate strong term matches
   - Negative scores suggest term absence/dissimilarity
   - Range varies based on document collection

2. Embedding Dot Product: Always produces positive scores
   - Higher values indicate semantic similarity
   - Range typically 0 to 1 after normalization

3. Score Normalization:
   - BM25: Normalized to [-1, 1] range preserving sign
   - Embeddings: Normalized to [0, 1] range
   - Maintains relative importance within each scoring method

4. Hybrid Scoring:
   - Currently uses simple averaging: (normalized_bm25 + normalized_embedding) / 2
   - Future plans include configurable weighting parameter
   - Additional tokenization options coming soon

This hybrid approach helps balance exact keyword matching (BM25) with semantic understanding (embeddings).

## Experimental

BM25 tokenization and ranking.

## License

MIT
