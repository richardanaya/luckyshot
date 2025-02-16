use crate::bm25_embedder::create_bm25_vector;
use crate::scan::FileEmbedding;
use bm25::{Embedding, ScoredDocument, Scorer, TokenEmbedding};

pub fn create_embedding_from_indices_and_values(
    indices: Vec<u32>,
    values: Vec<f32>,
) -> Embedding<usize> {
    let mut tokensEmbedddings: Vec<TokenEmbedding> = vec![];
    for (index, value) in indices.iter().zip(values.iter()) {
        let tokenEmbedding = TokenEmbedding {
            index: *index,
            value: *value,
        };
        tokensEmbedddings.push(tokenEmbedding);
    }
    Embedding(tokensEmbedddings)
}

pub fn rank_documents(embeddings: &[FileEmbedding], query: &str) -> Vec<ScoredDocument<usize>> {
    // Create scorer and add documents
    let mut scorer = Scorer::<usize>::new();

    // Add each document to the scorer with its index as ID
    for (i, e) in embeddings.iter().enumerate() {
        let mut emb =
            create_embedding_from_indices_and_values(e.bm25_indices.clone(), e.bm25_values.clone());
        scorer.upsert(&i, emb);
    }

    // Create query embedding
    let query_embedding = create_bm25_vector(query);
    let query_indices = query_embedding.0;
    let query_values = query_embedding.1;
    let query_embedding = create_embedding_from_indices_and_values(query_indices, query_values);

    // Get matches sorted by score
    scorer.matches(&query_embedding)
}
