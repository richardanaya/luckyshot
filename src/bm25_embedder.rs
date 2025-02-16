use bm25::{BM25, DefaultTokenizer, Language, Tokenizer};

pub fn create_bm25_vector(text: &str) -> Vec<f32> {
    // Create a default tokenizer with English settings
    let tokenizer = DefaultTokenizer::builder()
        .language_mode(Language::English)
        .normalization(true)
        .stopwords(true)
        .stemming(true)
        .build();

    // Tokenize the input text
    let terms = tokenizer.tokenize(text);

    // Create a BM25 instance
    let mut bm25 = BM25::default();

    // Add the document
    bm25.add_document(&terms);

    // Calculate BM25 scores for each unique term
    terms.iter()
        .map(|term| bm25.score(term) as f32)
        .collect()
}
