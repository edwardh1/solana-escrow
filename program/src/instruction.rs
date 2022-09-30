use solana_program::program_error::ProgramError;
use std::convert::TryInto;
use crate::error::EscrowError::InvalidInstruction;

pub enum EscrowInstruction {
    InitEscrow {
        amount: u64,
    },
    ReceiveTakerTokens {
        amount: u64,
    },
    InitializerWins {
        amount: u64,
    },
    TakerWins {
        amount: u64,
    },
}

impl EscrowInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::ReceiveTakerTokens {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::InitializerWins {
                amount: Self::unpack_amount(rest)?,
            },
            3 => Self::TakerWins {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
