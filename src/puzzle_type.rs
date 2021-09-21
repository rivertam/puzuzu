use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq)]
pub enum PuzzleType {
    Normal,
    Diagramless,
}

impl TryFrom<u16> for PuzzleType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0x0001 => Ok(PuzzleType::Normal),
            0x0401 => Ok(PuzzleType::Diagramless),
            _ => Err(()),
        }
    }
}
