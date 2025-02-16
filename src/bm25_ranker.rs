use crate::scan::FileEmbedding;
use bm25::{ScoredDocument, Scorer};
use crate::bm25_embedder::create_bm25_vector;

pub fn rank_documents(embeddings: &[FileEmbedding], query: &str) -> Vec<ScoredDocument<usize>> {
    // Create scorer and add documents
    let mut scorer = Scorer::<usize>::new();

    // Add each document to the scorer with its index as ID
    for (i, e) in embeddings.iter().enumerate() {
        scorer.upsert(&i, e.bm25_vector.clone().into());
    }

    // Create query embedding
    let query_embedding = create_bm25_vector(query);

    // Get matches sorted by score
    scorer.matches(&query_embedding.into())
}
