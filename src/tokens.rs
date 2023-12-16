#[derive(Debug, PartialEq)]
pub enum Token {
    KeyWord(String),
    Number(i32),
    Identifier(String),
    End,
}
