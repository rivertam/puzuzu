pub struct Square {}

impl Square {
    pub fn is_black_square(character: char) -> bool {
        character == '.' || character == ':'
    }

    pub fn black() -> char {
        '.'
    }
}
