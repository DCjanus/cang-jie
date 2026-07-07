use std::sync::Arc;

use cang_jie::{CangJieTokenizer, CangjieTokenStream, TokenizerOption};
use jieba_rs::Jieba;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

fn collect_tokens(mut tokenizer: CangJieTokenizer, text: &str) -> Vec<Token> {
    let mut stream = tokenizer.token_stream(text);
    let mut tokens = Vec::new();
    while stream.advance() {
        tokens.push(stream.token().clone());
    }
    tokens
}

#[test]
fn unicode_tokens_keep_byte_offsets() {
    let text = "南a京";
    let tokens = collect_tokens(
        CangJieTokenizer {
            worker: Arc::new(Jieba::empty()),
            option: TokenizerOption::Unicode,
        },
        text,
    );

    let actual = tokens
        .iter()
        .map(|token| (token.text.as_str(), token.offset_from, token.offset_to))
        .collect::<Vec<_>>();

    assert_eq!(actual, vec![("南", 0, 3), ("a", 3, 4), ("京", 4, 7)]);
}

#[test]
fn public_token_stream_constructor_keeps_slice_offsets() {
    let text = "南a京";
    let slices = vec![&text[..3], &text[3..4], &text[4..]];
    let mut stream = CangjieTokenStream::new(text, slices);

    let mut tokens = Vec::new();
    while stream.advance() {
        tokens.push(stream.token().clone());
    }

    let actual = tokens
        .iter()
        .map(|token| (token.text.as_str(), token.offset_from, token.offset_to))
        .collect::<Vec<_>>();

    assert_eq!(actual, vec![("南", 0, 3), ("a", 3, 4), ("京", 4, 7)]);
}

#[test]
#[should_panic(expected = "token slice must be borrowed from src")]
fn public_token_stream_constructor_rejects_unrelated_slices() {
    let src = "南京";
    let unrelated = "长江";

    let _ = CangjieTokenStream::new(src, vec![unrelated]);
}

#[test]
fn jieba_tokens_keep_valid_byte_offsets() {
    let text = "南京a长江";
    let options = [
        TokenizerOption::All,
        TokenizerOption::Default { hmm: false },
        TokenizerOption::ForSearch { hmm: false },
    ];

    for option in options {
        let tokens = collect_tokens(
            CangJieTokenizer {
                worker: Arc::new(Jieba::new()),
                option,
            },
            text,
        );

        assert!(!tokens.is_empty());
        for token in tokens {
            assert_eq!(
                token.text,
                &text[token.offset_from..token.offset_to],
                "token offsets should identify the emitted token text"
            );
        }
    }
}
