use crate::scan::FileEmbedding;
use bm25::{EmbedderBuilder, Language, ScoredDocument, Scorer};

pub fn rank_documents(embeddings: &[FileEmbedding], query: &str) -> Vec<ScoredDocument<usize>> {
    // Create scorer and add documents
    let mut scorer = Scorer::<usize>::new();

    // Add each document to the scorer with its index as ID
    for (i, e) in embeddings.iter().enumerate() {
        scorer.upsert(&i, e.bm25_vector);
    }

    // Create query embedding
    let query_embedding = create_bm25_vector(query);

    // Get matches sorted by score
    scorer.matches(&query_embedding)
}
