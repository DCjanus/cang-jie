use tantivy::tokenizer::Token;

#[derive(Debug)]
pub struct CangjieTokenStream<'a> {
    pub result: Vec<&'a str>,
    // Begin with 1
    pub index: usize,
    pub offset_from: usize,
    pub token: Token,
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
