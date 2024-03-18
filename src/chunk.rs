#[derive(Debug, PartialEq)]
pub enum OpCode {
    OpReturn = 0,
}

impl From<u8> for OpCode {
    fn from(opcode: u8) -> Self {
        match opcode {
            0 => OpCode::OpReturn,
            _ => unimplemented!("Not yet implemented!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::opcode_return(0, OpCode::OpReturn)]
    fn test_opcode_from_u8(#[case] from: u8, #[case] expected: OpCode) {
        assert_eq!(OpCode::from(from), expected)
    }
}
