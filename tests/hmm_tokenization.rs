use std::sync::Arc;

use cang_jie::{CangJieTokenizer, TokenizerOption};
use jieba_rs::Jieba;
use tantivy::tokenizer::{TokenStream, Tokenizer};

fn words(option: TokenizerOption, text: &str) -> Vec<String> {
    let mut tokenizer = CangJieTokenizer {
        worker: Arc::new(Jieba::new()),
        option,
    };
    let mut stream = tokenizer.token_stream(text);
    let mut words = Vec::new();
    while stream.advance() {
        words.push(stream.token().text.clone());
    }
    words
}

#[test]
fn hmm_controls_version_string_tokenization() {
    let text = "Python-3.12";

    assert_eq!(
        words(TokenizerOption::Default { hmm: false }, text),
        ["Python", "-", "3", ".", "12"]
    );
    assert_eq!(
        words(TokenizerOption::Default { hmm: true }, text),
        ["Python-3.12"]
    );
    assert_eq!(
        words(TokenizerOption::ForSearch { hmm: true }, text),
        ["Python", "Python-3.12"]
    );
}
