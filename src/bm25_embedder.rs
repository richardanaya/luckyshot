use bm25::{BM25Builder, BM25Parameters};

pub fn create_bm25_vector(text: &str) -> Vec<f32> {
    // Split text into terms (words)
    let terms: Vec<&str> = text.split_whitespace().collect();

    // Create a BM25 instance with default parameters
    let mut bm25 = BM25Builder::new()
        .with_parameters(BM25Parameters::default())
        .build();

    // Add the document
    bm25.add_document(&terms);

    // Get unique terms for scoring
    let unique_terms: Vec<&str> = terms.into_iter().collect();
    
    // Calculate BM25 scores for each term
    unique_terms.iter()
        .map(|&term| bm25.score(term) as f32)
        .collect()
}
