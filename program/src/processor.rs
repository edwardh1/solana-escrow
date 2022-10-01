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
            // EscrowInstruction::ReceiveTakerTokens { amount } => {
            //     msg!("Instruction: Exchange");
            //     Self::process_receive_taken_tokens(accounts, amount, program_id)
            // }
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
        let player_b_account = next_account_info(account_info_iter)?;
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
        // escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        // escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
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

    // fn process_receive_taken_tokens(
    //     accounts: &[AccountInfo],
    //     amount_expected_by_taker: u64,
    //     program_id: &Pubkey,
    // ) -> ProgramResult {
    //     let account_info_iter = &mut accounts.iter();
    //     let taker = next_account_info(account_info_iter)?;

    //     if !taker.is_signer {
    //         return Err(ProgramError::MissingRequiredSignature);
    //     }
    //     let takers_sending_token_account = next_account_info(account_info_iter)?;

    //     let takers_token_to_receive_account = next_account_info(account_info_iter)?;

    //     let pdas_temp_token_account = next_account_info(account_info_iter)?;
    //     let pdas_temp_token_account_info =
    //         TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
    //     let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

    //     if amount_expected_by_taker != pdas_temp_token_account_info.amount {
    //         return Err(EscrowError::ExpectedAmountMismatch.into());
    //     }
    //     let initializers_main_account = next_account_info(account_info_iter)?;
    //     let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    //     let escrow_account = next_account_info(account_info_iter)?;

    //     let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

    //     if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
    //         return Err(ProgramError::InvalidAccountData);
    //     }
    //     if escrow_info.initializer_pubkey != *initializers_main_account.key {
    //         return Err(ProgramError::InvalidAccountData);
    //     }
    //     if escrow_info.initializer_token_to_receive_account_pubkey
    //         != *initializers_token_to_receive_account.key
    //     {
    //         return Err(ProgramError::InvalidAccountData);
    //     }

    //     let token_program = next_account_info(account_info_iter)?;

    //     let transfer_to_initializer_ix = solana_program::instruction::transfer(
    //         takers_sending_token_account.key,
    //         initializers_token_to_receive_account.key,
    //         escrow_info.expected_amount,
    //     )?;
    //     msg!("Calling the token program to transfer tokens to the escrow's initializer...");
    //     invoke(
    //         &transfer_to_initializer_ix,
    //         &[
    //             takers_sending_token_account.clone(),
    //             initializers_token_to_receive_account.clone(),
    //             taker.clone(),
    //             token_program.clone(),
    //         ],
    //     )?;

        // let pda_account = next_account_info(account_info_iter)?;

        // let transfer_to_taker_ix = solana_program::instruction::transfer(
        //     pdas_temp_token_account.key,
        //     takers_token_to_receive_account.key,
        //     pdas_temp_token_account_info.amount,
        // )?;
        // msg!("Calling the token program to transfer tokens to the taker...");
        // invoke_signed(
        //     &transfer_to_taker_ix,
        //     &[
        //         pdas_temp_token_account.clone(),
        //         takers_token_to_receive_account.clone(),
        //         pda_account.clone(),
        //         token_program.clone(),
        //     ],
        //     &[&[&b"escrow"[..], &[nonce]]],
        // )?;

        // let close_pdas_temp_acc_ix = solana_program::instruction::close_account(
        //     token_program.key,
        //     pdas_temp_token_account.key,
        //     initializers_main_account.key,
        //     &pda,
        //     &[&pda],
        // )?;
        // msg!("Calling the token program to close pda's temp account...");
        // invoke_signed(
        //     &close_pdas_temp_acc_ix,
        //     &[
        //         pdas_temp_token_account.clone(),
        //         initializers_main_account.clone(),
        //         pda_account.clone(),
        //         token_program.clone(),
        //     ],
        //     &[&[&b"escrow"[..], &[nonce]]],
        // )?;

        // msg!("Closing the escrow account...");
        // **initializers_main_account.lamports.borrow_mut() = initializers_main_account
        //     .lamports()
        //     .checked_add(escrow_account.lamports())
        //     .ok_or(EscrowError::AmountOverflow)?;
        // **escrow_account.lamports.borrow_mut() = 0;

    //     Ok(())
    // }

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
