use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::Sysvar,
};

use crate::{
    error::AirdropError,
    instruction::deserialize_instruction_data,
    pda::{find_airdrop_user_data, find_mint_authority},
    state::{AirdropConfig, AirdropUserData},
    util::{
        process_airdrop_one_logic, process_initialize_airdrop_logic,
        process_initialize_airdrop_user_account_logic,
    },
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
        crate::instruction::AirdropInstruction::InitializeAirdropUser(_) => {
            process_initialize_airdrop_user(program_id, accounts)
        }
        crate::instruction::AirdropInstruction::MintOne(_) => {
            process_mint_one(program_id, accounts)
        }
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
    let admin_account = next_account_info(iter)?;
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

    // ----------------

    msg!("Get rent info from account");
    let rent = Rent::from_account_info(rent)?;

    process_initialize_airdrop_logic(
        airdrop_account,
        airdrop_authority,
        mint_authority,
        revenues_account,
        admin_account,
        fee_payer,
        airdrop_amount,
        metadata_prefix,
        symbol,
        price,
        program_id,
        rent,
        mint_authority_bump,
    )?;

    Ok(())
}

fn process_initialize_airdrop_user<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user_data_account = next_account_info(iter)?;
    let user = next_account_info(iter)?;
    let airdrop = next_account_info(iter)?;
    let rent = next_account_info(iter)?;
    let fee_payer = next_account_info(iter)?;

    // User data account checks
    msg!("Assert user data is properly derived");
    let (user_data_account_pda, user_data_account_bump) =
        find_airdrop_user_data(airdrop.key, user.key);

    if user_data_account_pda != *user_data_account.key {
        return Err(AirdropError::PdaCheckFailed.into());
    }

    msg!("Assert user data is not initialized");
    if user_data_account.lamports() > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    msg!("Assert user data account is writeable");
    assert_writeable(user_data_account)?;

    // User checks
    // msg!("Assert that user is regular wallet");
    // assert_owned_by(user, &system_program::id())?;

    // Airdrop config checks
    msg!("Assert that airdrop config is owned by program");
    assert_owned_by(airdrop, program_id)?;
    msg!("Assert that airdrop config is writeable");
    assert_writeable(airdrop)?;

    msg!("Assert airdrop config is initialized");
    let airdrop_data = AirdropConfig::unpack_from_account(airdrop)?;

    if !airdrop_data.is_initialized() {
        return Err(AirdropError::Uninitialized.into());
    }

    // Fee payer checks
    msg!("Assert that fee payer is signer");
    assert_signer(fee_payer)?;

    // ----------------

    msg!("Get rent");
    let rent = Rent::from_account_info(rent)?;

    process_initialize_airdrop_user_account_logic(
        user_data_account,
        user,
        airdrop,
        fee_payer,
        rent,
        program_id,
        user_data_account_bump,
    )?;

    Ok(())
}

fn process_mint_one<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let airdrop_config = next_account_info(iter)?;
    let user_data_account = next_account_info(iter)?;
    let mint_account = next_account_info(iter)?;
    let user = next_account_info(iter)?;
    let user_token_account = next_account_info(iter)?;
    let token_metadata_account = next_account_info(iter)?;
    let mint_authority = next_account_info(iter)?;
    let system_program = next_account_info(iter)?; // System program
    let clock_var = next_account_info(iter)?;
    let rent_var = next_account_info(iter)?;
    let token_program = next_account_info(iter)?; // Token program
    let _ = next_account_info(iter)?; // Associated token program
    let _ = next_account_info(iter)?; // Token metadata program
    let payer = next_account_info(iter)?;
    let airdrop_authority = next_account_info(iter)?;
    let admin_account = next_account_info(iter)?;
    let revenue_wallet = next_account_info(iter)?;

    // Airdrop config checks
    msg!("Check if airdrop account is writeable");
    assert_writeable(airdrop_config)?;
    msg!("Check if airdrop account is owned by this program");
    assert_owned_by(airdrop_config, program_id)?;

    let airdrop_data = AirdropConfig::unpack_from_account(airdrop_config)?;

    msg!("Check if airdrop account is initialized");
    if !airdrop_data.is_initialized() {
        return Err(AirdropError::Uninitialized.into());
    }

    msg!("Check supply");
    if airdrop_data.airdrop_index >= airdrop_data.airdrop_amount {
        return Err(AirdropError::OutOfSupply.into());
    }

    // User data account checks
    msg!("Assert user data is writeable");
    assert_writeable(user_data_account)?;
    msg!("Check if user data is owned by this program");
    assert_owned_by(user_data_account, program_id)?;

    let user_data = AirdropUserData::unpack_from_account(user_data_account)?;

    msg!("Check if user data account is initialized");
    if !user_data.is_initialized() {
        return Err(AirdropError::Uninitialized.into());
    }

    msg!("Check if airdrop data and user wallet are valid for user data account");
    if !(user_data.user == *user.key && user_data.airdrop == *airdrop_config.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let clock = Clock::from_account_info(clock_var)?;

    msg!("Check user timeout");
    if user_data.locked_till >= clock.unix_timestamp as u64 {
        return Err(AirdropError::UserTimeout.into());
    }

    // Mint account checks
    msg!("Assert that mint account is signer");
    assert_signer(mint_account)?;
    msg!("Assert that mint account is writeable");
    assert_writeable(mint_account)?;

    // User token account checks
    msg!("Assert token account is writeable");
    assert_writeable(user_token_account)?;

    // Metadata account checks
    msg!("Assert metadata account is writeable");
    assert_writeable(token_metadata_account)?;

    // Mint authority checks
    let (mint_authority_pda, mint_authority_bump) = find_mint_authority(airdrop_config.key);

    msg!("Assert mint authority is properly derived");
    if mint_authority_pda != *mint_authority.key {
        return Err(AirdropError::PdaCheckFailed.into());
    }

    // Payer checks
    msg!("Assert payer is signer");
    assert_signer(payer)?;
    msg!("Assert payer is writeable");
    assert_writeable(payer)?;
    msg!("Assert payer is owned by system program");
    assert_owned_by(payer, &system_program::id())?;

    // Airdrop authority checks
    msg!("Assert drop is approved by airdrop authority");
    assert_signer(airdrop_authority)?;

    // Admin account checks
    msg!("Assert that admin account is correct one");
    if airdrop_data.admin_account != *admin_account.key {
        return Err(AirdropError::WrongAccountAddress.into());
    }

    // Revenue wallet checks
    msg!("Assert that revenue wallet is correct one");
    if airdrop_data.revenues_wallet != *revenue_wallet.key {
        return Err(AirdropError::WrongAccountAddress.into());
    }

    msg!("Assert revenue wallet is writeable");
    assert_writeable(revenue_wallet)?;

    // ----------------

    let rent = Rent::from_account_info(rent_var)?;

    process_airdrop_one_logic(
        airdrop_config,
        user_data_account,
        mint_account,
        user,
        user_token_account,
        token_metadata_account,
        mint_authority,
        rent_var,
        clock,
        rent,
        payer,
        admin_account,
        revenue_wallet,
        mint_authority_bump,
        system_program,
        token_program,
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
