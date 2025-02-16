use crate::tokenizer::get_tokenizer;
use bm25::{DefaultTokenizer, EmbedderBuilder};

#[derive(Debug)]
pub struct Bm25Vector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

pub fn create_bm25_vector(text: &str, avgdl: f32) -> Bm25Vector {
    let mut embedder_builder = EmbedderBuilder::<u32, DefaultTokenizer>::with_avgdl(avgdl);
    embedder_builder = embedder_builder.tokenizer(get_tokenizer());
    embedder_builder = embedder_builder.b(0.0);
    let embedder = embedder_builder.build();

    let embedding = embedder.embed(text);
    Bm25Vector {
        indices: embedding.indices().cloned().collect(),
        values: embedding.values().cloned().collect(),
    }
}
