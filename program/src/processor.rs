use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account as TokenAccount;
use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};

pub struct EscrowProcessor;
impl EscrowProcessor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }
            EscrowInstruction::PlayerAWins { amount } => {
                msg!("Instruction: Player A");
                Self::process_player_a_wins(accounts, amount, program_id)
            }
            EscrowInstruction::PlayerBWins { amount } => {
                msg!("Instruction: Player B");
                Self::process_player_b_wins(accounts, amount, program_id)
            }
        }
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let player_a_account = next_account_info(account_info_iter)?;
        if player_a_account.lamports() < amount {
            return Err(EscrowError::InsufficientLamports.into());
        }
        let player_b_account = next_account_info(account_info_iter)?;
        if player_b_account.lamports() < amount {
            return Err(EscrowError::InsufficientLamports.into());
        }

        let receiver_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *receiver_account.key;
        escrow_info.expected_amount = amount;

        let transfer_to_initializer_ix = solana_program::system_instruction::transfer(
            player_a_account.key,
            receiver_account.key,
            escrow_info.expected_amount,
        );
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                player_a_account.clone(),
                receiver_account.clone(),
            ],
        )?;

        let transfer_to_initializer_ix = solana_program::system_instruction::transfer(
            player_b_account.key,
            receiver_account.key,
            escrow_info.expected_amount,
        );
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                player_b_account.clone(),
                receiver_account.clone(),
            ],
        )?;
        Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_player_a_wins(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let player_a_account = next_account_info(account_info_iter)?;
        let player_b_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;
        let transfer_to_initializer_ix = solana_program::system_instruction::transfer(
            player_a_account.key,
            receiver_account.key,
            escrow_info.expected_amount,
        );
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                player_a_account.clone(),
                receiver_account.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_player_b_wins(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let player_a_account = next_account_info(account_info_iter)?;
        let player_b_account = next_account_info(account_info_iter)?;
        let receiver_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

        let transfer_to_initializer_ix = solana_program::system_instruction::transfer(
            player_b_account.key,
            receiver_account.key,
            escrow_info.expected_amount,
        );
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                player_b_account.clone(),
                receiver_account.clone(),
            ],
        )?;
        Ok(())
    }
}
