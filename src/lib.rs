extern crate jieba_rs;
#[macro_use]
extern crate lazy_static;
extern crate tantivy;

pub use stream::CangjieTokenStream;
pub use tokenizer::CangJieTokenizer;

pub mod stream;
pub mod tokenizer;
