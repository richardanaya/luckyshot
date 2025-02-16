pub fn bm25_similarity(query: &[f32], doc: &[f32]) -> f32 {
    const K1: f32 = 1.5;  // Term frequency saturation parameter
    const B: f32 = 0.75;  // Length normalization parameter
    const EPSILON: f32 = 1e-10;  // Small value to prevent division by zero

    // Calculate average document length (in this case, it's always the embedding dimension)
    let avg_dl = doc.len() as f32;
    let doc_len = doc.len() as f32;

    // Calculate BM25 score
    let mut score = 0.0;
    for (q, d) in query.iter().zip(doc.iter()) {
        // Treat the embedding components as term frequencies
        let tf = d.abs();
        
        // IDF-like component using the query values
        let idf = (q.abs() + EPSILON).ln();
        
        // BM25 formula
        let numerator = tf * (K1 + 1.0);
        let denominator = tf + K1 * (1.0 - B + B * doc_len / avg_dl);
        
        score += idf * numerator / denominator;
    }

    score
}
