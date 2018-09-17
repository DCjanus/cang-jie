extern crate jieba_rs;
#[macro_use]
extern crate lazy_static;
extern crate tantivy;

use jieba_rs::Jieba;
use tantivy::tokenizer::Token;

lazy_static! {
    pub static ref jieba: Jieba = ::jieba_rs::Jieba::default();
}

pub fn init() {
    jieba.cut_all("");
}

#[derive(Default, Clone, Debug)]
pub struct CangJieTokenizer;

impl<'a> ::tantivy::tokenizer::Tokenizer<'a> for CangJieTokenizer {
    type TokenStreamImpl = CangjieTokenStream<'a>;

    fn token_stream(&self, text: &'a str) -> Self::TokenStreamImpl {
        let result = jieba.cut_all(text);
        CangjieTokenStream {
            result,
            index: 0,
            offset_from: 0,
            token: Token::default(),
        }
    }
}

#[derive(Debug)]
pub struct CangjieTokenStream<'a> {
    result: Vec<&'a str>,
    // Begin with 1
    index: usize,
    offset_from: usize,
    token: Token,
}

impl<'a> ::tantivy::tokenizer::TokenStream for CangjieTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.index < self.result.len() {
            self.index += 1;

            let current_word = self.result[self.index - 1];

            self.offset_from += current_word.bytes().len();

            self.token = Token {
                offset_from: self.offset_from,
                offset_to: self.offset_from + current_word.bytes().len(),
                position: self.index,
                text: current_word.to_string(),
                position_length: self.result.len(),
            };
            true
        } else {
            false
        }
    }

    fn token(&self) -> &::tantivy::tokenizer::Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut ::tantivy::tokenizer::Token {
        &mut self.token
    }
}
