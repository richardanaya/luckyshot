use crate::bm25_embedder::{create_bm25_vector, Bm25Vector};
use crate::scan::FileEmbedding;
use bm25::{Embedding, ScoredDocument, Scorer, TokenEmbedding};

pub fn create_embedding_from_indices_and_values(
    indices: Vec<u32>,
    values: Vec<f32>,
) -> Embedding<u32> {
    let mut token_embeddings: Vec<TokenEmbedding<u32>> = vec![];
    for (index, value) in indices.iter().zip(values.iter()) {
        let token_embedding = TokenEmbedding {
            index: *index,
            value: *value,
        };
        token_embeddings.push(token_embedding);
    }
    Embedding(token_embeddings)
}

pub fn rank_documents(embeddings: &[FileEmbedding], query: &str) -> Vec<ScoredDocument<u32>> {
    // Create scorer and add documents
    let mut scorer = Scorer::<u32>::new();

    // Add each document to the scorer with its index as ID
    for (i, e) in embeddings.iter().enumerate() {
        let emb =
            create_embedding_from_indices_and_values(e.bm25_indices.clone(), e.bm25_values.clone());
        scorer.upsert(&(i as u32), emb);
    }

    // Create query embedding
    let query_embedding = create_bm25_vector(query);
    let query_embedding = create_embedding_from_indices_and_values(
        query_embedding.indices,
        query_embedding.values,
    );

    // Get matches sorted by score
    scorer.matches(&query_embedding)
}
