use std::sync::OnceLock;

pub struct Language<'b> {
    words_buffer: &'b str,
    prefix_length: usize,
    words: OnceLock<Vec<&'b str>>,
    truncated_words: OnceLock<Vec<&'b str>>,
}

impl<'b> Language<'b> {
    pub const fn new(buffer: &'b str, prefix_length: usize) -> Self {
        Self {
            words_buffer: buffer,
            prefix_length,
            words: OnceLock::new(),
            truncated_words: OnceLock::new(),
        }
    }

    pub fn prefix_length(&self) -> usize {
        self.prefix_length
    }

    pub fn words(&self) -> &[&'b str] {
        self.words
            .get_or_init(move || self.words_buffer.split(',').collect())
            .as_slice()
    }

    pub fn truncated_words(&self) -> &[&'b str] {
        self.truncated_words
            .get_or_init(|| {
                self.words()
                    .iter()
                    .map(|s| &s[..self.prefix_length.min(s.len())])
                    .collect()
            })
            .as_slice()
    }
}
