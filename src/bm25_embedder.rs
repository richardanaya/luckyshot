use bm25::{DefaultTokenizer, Language, Tokeniz};

pub fn create_bm25_vector(text: &str) -> Vec<u32> {
    let embedder = EmbedderBuilder::<u32, DefaultTokenizer>::with_avgdl(1.0).build();
    let embedding = embedder.embed(text);
    embedding.indices().cloned().collect::<Vec<_>>()
}
