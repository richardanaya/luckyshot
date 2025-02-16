use bm25::{EmbedderBuilder, Language, Scorer, ScoredDocument};
use crate::scan::FileEmbedding;

pub fn rank_documents(embeddings: &[FileEmbedding], query: &str) -> Vec<ScoredDocument<usize>> {
    // Create a corpus from the documents
    let corpus: Vec<String> = embeddings.iter()
        .map(|e| e.filename.clone())
        .collect();

    // Create embedder with English language settings
    let embedder = EmbedderBuilder::with_fit_to_corpus(Language::English, &corpus).build();

    // Create scorer and add documents
    let mut scorer = Scorer::<usize>::new();
    
    // Add each document to the scorer with its index as ID
    for (i, _) in corpus.iter().enumerate() {
        let document_embedding = embedder.embed(&corpus[i]);
        scorer.upsert(&i, document_embedding);
    }

    // Create query embedding
    let query_embedding = embedder.embed(query);

    // Get matches sorted by score
    scorer.matches(&query_embedding)
}
