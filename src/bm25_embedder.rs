use bm25::{DefaultTokenizer, Tokenizer, EmbedderBuilder};

pub fn create_bm25_vector(text: &str) -> Vec<f32> {
    let embedder = EmbedderBuilder::<f32, DefaultTokenizer>::with_avgdl(1.0).build();
    let embedding = embedder.embed(text);
    embedding.indices().cloned().collect::<Vec<_>>()
}
