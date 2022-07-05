use metaplex_token_metadata::state::Creator;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
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
    admin_account: &AccountInfo,
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
        admin_account: *admin_account.key,
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

pub fn process_airdrop_one_logic<'a>(
    airdrop_config: &'a AccountInfo<'a>,
    user_data_account: &'a AccountInfo<'a>,
    mint: &'a AccountInfo<'a>,
    user: &'a AccountInfo<'a>,
    user_token_account: &'a AccountInfo<'a>,
    metadata: &'a AccountInfo<'a>,
    mint_authority: &'a AccountInfo<'a>,
    rent_account: &'a AccountInfo<'a>,
    clock: Clock,
    rent: Rent,
    payer: &'a AccountInfo<'a>,
    admin: &'a AccountInfo<'a>,
    revenue_wallet: &'a AccountInfo<'a>,
    mint_authority_bump: u8,
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    // Create mint account for token
    let lamports = rent.minimum_balance(spl_token::state::Account::LEN);

    msg!("Initialize account for mint");
    // Create mint
    invoke(
        &system_instruction::create_account(
            payer.key,
            mint.key,
            lamports,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[payer.clone(), mint.clone()],
    )?;

    msg!("Fill mint data");
    // Initialize mint
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint.key,
            mint_authority.key,
            None,
            0,
        )?,
        &[mint.clone(), rent_account.clone()],
    )?;

    msg!("Initialize user token account");
    // Initialize user token account
    invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer.key, user.key, mint.key,
        ),
        &[
            payer.clone(),
            user_token_account.clone(),
            user.clone(),
            mint.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    let mut airdrop_data = AirdropConfig::unpack_from_account(airdrop_config)?;
    let symbol_str = str_from_u8_nul_utf8(&airdrop_data.symbol)
        .or(Err(ProgramError::InvalidAccountData))?
        .to_string();

    let uri_prefix = str_from_u8_nul_utf8(&airdrop_data.metadata_prefix)
        .or(Err(ProgramError::InvalidAccountData))?
        .to_string();

    let creators = vec![
        Creator {
            address: *mint_authority.key,
            verified: true,
            share: 0,
        },
        Creator {
            address: *revenue_wallet.key,
            verified: false,
            share: 100,
        },
    ];

    let mint_authority_seed = &[
        MINT_AUTHORITY.as_bytes(),
        airdrop_config.key.as_ref(),
        &[mint_authority_bump],
    ];

    msg!("Initialize metadata");
    // Create token metadata
    invoke_signed(
        &metaplex_token_metadata::instruction::create_metadata_accounts(
            metaplex_token_metadata::id(),
            *metadata.key,
            *mint.key,
            *mint_authority.key,
            *payer.key,
            *mint_authority.key,
            symbol_str.clone() + &format!(" #{}", airdrop_data.airdrop_index),
            symbol_str,
            uri_prefix + &format!("{}.json", airdrop_data.airdrop_index),
            Some(creators),
            1000,
            false,
            true,
        ),
        &[
            metadata.clone(),
            mint.clone(),
            mint_authority.clone(),
            payer.clone(),
            mint_authority.clone(),
            system_program.clone(),
            rent_account.clone(),
        ],
        &[mint_authority_seed],
    )?;

    msg!("Mint to user");
    // Mint one token to user
    invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint.key,
            user_token_account.key,
            mint_authority.key,
            &[],
            1,
        )?,
        &[
            mint.clone(),
            user_token_account.clone(),
            mint_authority.clone(),
        ],
        &[mint_authority_seed],
    )?;

    msg!("Update metadata");
    // Mark NFT as sold and transfer update authority
    invoke_signed(
        &metaplex_token_metadata::instruction::update_metadata_accounts(
            metaplex_token_metadata::id(),
            *metadata.key,
            *mint_authority.key,
            Some(*admin.key),
            None,
            Some(true),
        ),
        &[metadata.clone(), mint_authority.clone()],
        &[mint_authority_seed],
    )?;

    msg!("Revoke mint authority");
    // Revoke mint authority
    invoke_signed(
        &spl_token::instruction::set_authority(
            &spl_token::id(),
            mint.key,
            None,
            spl_token::instruction::AuthorityType::MintTokens,
            mint_authority.key,
            &[],
        )?,
        &[mint.clone(), mint_authority.clone()],
        &[mint_authority_seed],
    )?;

    msg!("Transfer SOL");
    // Transfer SOL to revenue wallet
    invoke(
        &system_instruction::transfer(payer.key, revenue_wallet.key, airdrop_data.price),
        &[payer.clone(), revenue_wallet.clone()],
    )?;

    msg!("Write changes to program accounts");
    airdrop_data.airdrop_index += 1;
    AirdropConfig::pack_into_account(airdrop_data, airdrop_config)?;
    let mut user_data = AirdropUserData::unpack_from_account(user_data_account)?;
    user_data.mints_amount += 1;
    user_data.locked_till = (clock.unix_timestamp + 21600_i64) as u64; // Current time plus 6 hours
    AirdropUserData::pack_into_account(user_data, user_data_account)?;

    Ok(())
}

fn str_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<&str, std::str::Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8(&utf8_src[0..nul_range_end])
}
