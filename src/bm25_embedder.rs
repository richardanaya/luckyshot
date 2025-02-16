use bm25::BM25;

pub fn create_bm25_vector(text: &str) -> Vec<f32> {
    // Split text into terms (words)
    let terms: Vec<String> = text
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    // Create a BM25 instance
    let mut bm25 = BM25::default();

    // Add the document
    bm25.add_document(&terms);

    // Get unique terms for scoring
    let unique_terms: Vec<String> = terms.into_iter().collect();
    
    // Calculate BM25 scores for each term
    unique_terms.iter()
        .map(|term| bm25.score(term) as f32)
        .collect()
}
