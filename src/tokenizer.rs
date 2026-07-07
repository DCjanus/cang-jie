use crate::{
    options::TokenizerOption,
    stream::{CangjieToken, CangjieTokenStream},
};
use jieba_rs::Jieba;
use log::trace;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CangJieTokenizer {
    /// Separation algorithm provider
    pub worker: Arc<Jieba>,
    /// Separation config
    pub option: TokenizerOption,
}

impl Default for CangJieTokenizer {
    fn default() -> Self {
        CangJieTokenizer {
            worker: Arc::new(Jieba::empty()),
            option: TokenizerOption::Default { hmm: false },
        }
    }
}

impl ::tantivy::tokenizer::Tokenizer for CangJieTokenizer {
    type TokenStream<'a> = CangjieTokenStream<'a>;

    /// Cut text into tokens
    fn token_stream<'a>(&mut self, text: &'a str) -> CangjieTokenStream<'a> {
        let result = match self.option {
            TokenizerOption::All => self
                .worker
                .cut_all(text)
                .into_iter()
                .map(|token| CangjieToken::new(token.word, token.byte_start, token.byte_end))
                .collect(),
            TokenizerOption::Default { hmm: use_hmm } => self
                .worker
                .cut(text, use_hmm)
                .into_iter()
                .map(|token| CangjieToken::new(token.word, token.byte_start, token.byte_end))
                .collect(),
            TokenizerOption::ForSearch { hmm: use_hmm } => self
                .worker
                .cut_for_search(text, use_hmm)
                .into_iter()
                .map(|token| CangjieToken::new(token.word, token.byte_start, token.byte_end))
                .collect(),
            TokenizerOption::Unicode => {
                text.chars()
                    .fold((0usize, vec![]), |(offset, mut tokens), the_char| {
                        let byte_end = offset + the_char.len_utf8();
                        tokens.push(CangjieToken::new(&text[offset..byte_end], offset, byte_end));
                        (byte_end, tokens)
                    })
                    .1
            }
        };
        trace!("{:?}->{:?}", text, result);
        CangjieTokenStream::from_tokens(result)
    }
}
