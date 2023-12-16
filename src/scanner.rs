use crate::tokens::Token;

pub struct TokenIterator {
    input: String,
    eof: bool,
}

impl Iterator for TokenIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        const SKIP: &str = r"([\s]*(//.*\n)?)*";
        const GROUP1: &str = r"while\b|do\b|if\b|then\b|else\b|:=|[;{}<=+\-*/]";
        const GROUP2: &str = r"[A-Za-z_][A-Za-z_0-9]*";
        const GROUP3: &str = r"[0-9]+";

        let regexp = format!(r"{}(({})|({})|({}))", SKIP, GROUP1, GROUP2, GROUP3);
        let pattern = regex::Regex::new(&regexp).ok()?;

        if !self.eof && self.input.trim().is_empty() {
            self.eof = true;
            return Some(Token::End);
        }

        if let Some(cap) = pattern.captures(&self.input.clone()) {
            let matched_length = cap.get(0).unwrap().end();
            self.input = self.input[matched_length..]
                .to_string()
                .trim_start()
                .to_string();

            if let Some(s) = cap.get(4).map(|m| m.as_str()) {
                return Some(Token::KeyWord(s.to_string()));
            }
            if let Some(s) = cap.get(5).map(|m| m.as_str()) {
                return Some(Token::Identifier(s.to_string()));
            }
            if let Some(s) = cap.get(6).map(|m| m.as_str()) {
                return Some(Token::Number(s.parse::<i32>().ok()?));
            }
        }
        None
    }
}

pub fn tokenize(input: String) -> TokenIterator {
    TokenIterator { input, eof: false }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{scanner::tokenize, tokens::Token};

    #[test]
    fn test_last_token_kind() -> Result<()> {
        let sample = "
        {
            i := 10;
            while 0 < i do
              i := i - 1
        }
        ";

        let mut iter = tokenize(sample.to_string());
        let mut last = None;
        while let Some(token) = iter.next() {
            last = Some(token);
        }
        assert_eq!(last, Some(Token::End));
        Ok(())
    }

    #[test]
    fn test_second_to_last_token_kind() -> Result<()> {
        let sample = "
        {
            i := 10;
            while 0 < i do
              i := i - 1
        }
        ";

        let mut iter = tokenize(sample.to_string());
        let mut last = None;
        let mut second_to_last = None;
        while let Some(token) = iter.next() {
            second_to_last = last;
            last = Some(token);
        }
        assert_eq!(second_to_last, Some(Token::KeyWord("}".to_string())));
        Ok(())
    }

    #[test]
    fn test_comments() -> Result<()> {
        let sample2 = "
        {
            i := 10;        // this is comment.
            while i do
              i := i - 1
        }
        ";

        let mut iter = tokenize(sample2.to_string());
        let mut last = None;
        let mut second_to_last = None;
        while let Some(token) = iter.next() {
            second_to_last = last;
            last = Some(token);
        }
        assert_eq!(second_to_last, Some(Token::KeyWord("}".to_string())));
        assert_eq!(last, Some(Token::End));
        Ok(())
    }
}
