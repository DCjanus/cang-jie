use tantivy::tokenizer::Token;

#[derive(Debug)]
pub(crate) struct CangjieToken<'a> {
    pub word: &'a str,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl<'a> CangjieToken<'a> {
    pub fn new(word: &'a str, byte_start: usize, byte_end: usize) -> Self {
        CangjieToken {
            word,
            byte_start,
            byte_end,
        }
    }
}

#[derive(Debug)]
pub struct CangjieTokenStream<'a> {
    result: Vec<CangjieToken<'a>>,
    // Begin with 1
    index: usize,
    token: Token,
}

impl<'a> CangjieTokenStream<'a> {
    /// Create a token stream from slices borrowed from `src`.
    ///
    /// Every item in `result` must be a subslice of `src` so byte offsets can be
    /// derived from the slice addresses.
    pub fn new(src: &'a str, result: Vec<&'a str>) -> Self {
        let base = src.as_ptr() as usize;
        let end = base + src.len();
        let result = result
            .into_iter()
            .map(|word| {
                let word_start = word.as_ptr() as usize;
                let word_end = word_start + word.len();
                assert!(
                    base <= word_start && word_end <= end,
                    "token slice must be borrowed from src"
                );
                let byte_start = word_start - base;
                CangjieToken::new(word, byte_start, byte_start + word.len())
            })
            .collect();
        Self::from_tokens(result)
    }

    pub(crate) fn from_tokens(result: Vec<CangjieToken<'a>>) -> Self {
        CangjieTokenStream {
            result,
            index: 0,
            token: Token::default(),
        }
    }
}

impl<'a> ::tantivy::tokenizer::TokenStream for CangjieTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.index < self.result.len() {
            let current_token = &self.result[self.index];

            self.token = Token {
                offset_from: current_token.byte_start,
                offset_to: current_token.byte_end,
                position: self.index,
                text: current_token.word.to_string(),
                position_length: 1,
            };

            self.index += 1;
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
