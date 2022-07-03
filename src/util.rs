use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

use crate::state::{AirdropConfig, AirdropUserData, MintAuthority, MINT_AUTHORITY, USER_DATA};

pub fn process_initialize_airdrop_logic<'a>(
    airdrop_account: &AccountInfo,
    airdrop_authority: &AccountInfo,
    mint_authority: &'a AccountInfo<'a>,
    revenues_account: &AccountInfo,
    fee_payer: &'a AccountInfo<'a>,
    airdrop_amount: u64,
    metadata_prefix: [u8; 32],
    symbol: [u8; 8],
    price: u64,
    program_id: &Pubkey,
    rent: Rent,
    mint_authority_bump: u8,
) -> ProgramResult {
    let airdrop_data = AirdropConfig::unpack_from_account(airdrop_account)?;

    msg!("Check if airdrop data already initialized");
    if airdrop_data.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let airdrop_data = AirdropConfig {
        initialized: true,
        airdrop_authority: *airdrop_authority.key,
        airdrop_index: 0,
        airdrop_amount,
        metadata_prefix,
        symbol,
        airdrop_users: 0,
        revenues_wallet: *revenues_account.key,
        price,
    };

    AirdropConfig::pack_into_account(airdrop_data, airdrop_account)?;

    let lamports = rent.minimum_balance(MintAuthority::LEN);
    let mint_authority_seed = &[
        MINT_AUTHORITY.as_bytes(),
        airdrop_account.key.as_ref(),
        &[mint_authority_bump],
    ];

    msg!("Initialize mint authority");
    invoke_signed(
        &system_instruction::create_account(
            fee_payer.key,
            mint_authority.key,
            lamports,
            MintAuthority::LEN as u64,
            program_id,
        ),
        &[fee_payer.clone(), mint_authority.clone()],
        &[mint_authority_seed],
    )?;

    Ok(())
}

pub fn process_initialize_airdrop_user_account_logic<'a>(
    user_data_account: &'a AccountInfo<'a>,
    user: &'a AccountInfo<'a>,
    airdrop_config: &'a AccountInfo<'a>,
    fee_payer: &'a AccountInfo<'a>,
    rent: Rent,
    program_id: &Pubkey,
    user_data_account_bump: u8,
) -> ProgramResult {

    // Create account
    msg!("Initialize user airdrop account");
    let lamports = rent.minimum_balance(AirdropUserData::LEN);
    let user_data_account_seed = &[
        USER_DATA.as_bytes(),
        airdrop_config.key.as_ref(),
        user.key.as_ref(),
        &[user_data_account_bump],
    ];

    invoke_signed(
        &system_instruction::create_account(
            fee_payer.key,
            user_data_account.key,
            lamports,
            AirdropUserData::LEN as u64,
            program_id,
        ),
        &[fee_payer.clone(), user_data_account.clone()],
        &[user_data_account_seed],
    )?;

    // Write account data
    let user_account_data = AirdropUserData {
        initialized: true,
        airdrop: *airdrop_config.key,
        user: *user.key,
        mints_amount: 0,
        locked_till: 0,
    };

    AirdropUserData::pack_into_account(user_account_data, user_data_account)?;

    // Increase user counter
    let mut airdrop_config_data = AirdropConfig::unpack_from_account(airdrop_config)?;
    airdrop_config_data.airdrop_users += 1;
    AirdropConfig::pack_into_account(airdrop_config_data, airdrop_config)?;


    todo!()
}
