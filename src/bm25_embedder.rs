use bm25::{EmbedderBuilder, Tokenizer};
use crate::tokenize_code::tokenize_code;

#[derive(Debug)]
pub struct Bm25Vector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

#[derive(Default)]
struct CodeTokenizer;

// Tokenize on occurrences of "T"
impl Tokenizer for CodeTokenizer {
    fn tokenize(&self, input_text: &str) -> Vec<String> {
        tokenize_code(input_text)
    }
}

pub fn create_bm25_vector(text: &str) -> Bm25Vector {
    let embedder = EmbedderBuilder::<u32, CodeTokenizer>::with_avgdl(250.0).build();
    let embedding = embedder.embed(text);
    Bm25Vector {
        indices: embedding.indices().cloned().collect(),
        values: embedding.values().cloned().collect(),
    }
}
