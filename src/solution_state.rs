use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq)]
pub enum SolutionState {
    /// solution is available in plaintext
    Unlocked,
    /// solution is locked (scrambled) with a key
    Locked,
}

impl TryFrom<u16> for SolutionState {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0x0000 => Ok(SolutionState::Unlocked),
            0x0004 => Ok(SolutionState::Locked),
            _ => Err(()),
        }
    }
}
