use bm25::{DefaultTokenizer, EmbedderBuilder};

#[derive(Debug)]
pub struct Bm25Vector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

pub fn create_bm25_vector(text: &str) -> Bm25Vector {
    let embedder = EmbedderBuilder::<u32, DefaultTokenizer>::with_avgdl(1.0).build();
    let embedding = embedder.embed(text);
    Bm25Vector {
        indices: embedding.indices().cloned().collect(),
        values: embedding.values().cloned().collect(),
    }
}
