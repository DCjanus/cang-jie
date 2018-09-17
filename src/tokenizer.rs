use jieba_rs::Jieba;
use stream::CangjieTokenStream;
use tantivy::tokenizer::Token;

lazy_static! {
    pub static ref jieba: Jieba = ::jieba_rs::Jieba::default();
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
