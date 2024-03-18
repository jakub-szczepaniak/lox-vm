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

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
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

    #[rstest]
    #[case::opcode_return(OpCode::OpReturn, 0)]
    fn test_byte_from_opcode(#[case] from: OpCode, #[case] expected: u8) {
        assert_eq!(u8::from(from), expected)
    }
}
