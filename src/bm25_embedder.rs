use crate::tokenize_code::tokenize_code;
use bm25::{EmbedderBuilder, Tokenizer};

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
        tokenize_code(input_text, "input.rs")
    }
}

pub fn create_bm25_vector(text: &str, avgdl: f32) -> Bm25Vector {
    let embedder = EmbedderBuilder::<u32, CodeTokenizer>::with_avgdl(avgdl).build();
    let embedding = embedder.embed(text);
    Bm25Vector {
        indices: embedding.indices().cloned().collect(),
        values: embedding.values().cloned().collect(),
    }
}
