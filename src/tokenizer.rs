use bm25::{DefaultTokenizer, Language};

pub fn get_tokenizer() -> DefaultTokenizer {
    DefaultTokenizer::builder()
        .language_mode(Language::English)
        .normalization(true) // Normalize unicode (e.g., 'é' -> 'e', '🍕' -> 'pizza', etc.)
        .stopwords(true) // Remove common words with little meaning (e.g., 'the', 'and', 'of', etc.)
        .stemming(true) // Reduce words to their root form (e.g., 'running' -> 'run')
        .build()
}
