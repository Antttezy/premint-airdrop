use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

use crate::{
    error::AirdropError,
    instruction::deserialize_instruction_data,
    pda::find_mint_authority,
    util::process_initialize_airdrop_logic,
};

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    match deserialize_instruction_data(instruction_data)? {
        crate::instruction::AirdropInstruction::InitializeAirdrop(args) => {
            process_initialize_airdrop(
                program_id,
                accounts,
                args.airdrop_amount,
                args.metadata_prefix,
                args.symbol,
                args.price,
            )
        }
        crate::instruction::AirdropInstruction::InitializeAirdropUser(_) => todo!(),
    }
}

fn process_initialize_airdrop<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    airdrop_amount: u64,
    metadata_prefix: [u8; 32],
    symbol: [u8; 8],
    price: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let airdrop_account = next_account_info(iter)?;
    let airdrop_authority = next_account_info(iter)?;
    let mint_authority = next_account_info(iter)?;
    let revenues_account = next_account_info(iter)?;
    let rent = next_account_info(iter)?;
    let fee_payer = next_account_info(iter)?;

    // Airdrop account checks
    msg!("Assert airdrop config writeable");
    assert_writeable(airdrop_account)?;
    msg!("Assert airdrop config owned by program");
    assert_owned_by(airdrop_account, program_id)?;

    // Airdrop authority checks

    // Mint authority checks
    let (mint_authority_pda, mint_authority_bump) = find_mint_authority(airdrop_account.key);

    msg!("Assert mint authority is PDA");
    if mint_authority_pda != *mint_authority.key {
        return Err(AirdropError::PdaCheckFailed.into());
    }

    // Revenues account checks

    // Fee payer checks
    msg!("Assert fee payer is signer");
    assert_signer(fee_payer)?;

    msg!("Get rent info from account");
    let rent = Rent::from_account_info(rent)?;

    process_initialize_airdrop_logic(
        airdrop_account,
        airdrop_authority,
        mint_authority,
        revenues_account,
        fee_payer,
        airdrop_amount,
        metadata_prefix,
        symbol,
        price,
        program_id,
        rent,
        mint_authority_bump
    )?;

    Ok(())
}

fn assert_signer(acc: &AccountInfo) -> Result<(), ProgramError> {
    match acc.is_signer {
        true => Ok(()),
        false => Err(AirdropError::SignerRequired.into()),
    }
}

fn assert_writeable(acc: &AccountInfo) -> Result<(), ProgramError> {
    match acc.is_writable {
        true => Ok(()),
        false => Err(AirdropError::WriteableRequired.into()),
    }
}

fn assert_owned_by(acc: &AccountInfo, expected_owner: &Pubkey) -> Result<(), ProgramError> {
    match acc.owner.eq(expected_owner) {
        true => Ok(()),
        false => Err(ProgramError::IllegalOwner),
    }
}
