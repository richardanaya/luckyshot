pub fn dot_product_similarity(query: &[f32], doc: &[f32]) -> f32 {
    query.iter()
        .zip(doc.iter())
        .map(|(a, b)| a * b)
        .sum()
}
