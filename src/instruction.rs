use std::convert::TryInto;

use arrayref::array_refs;
use solana_program::program_error::ProgramError;

use crate::error::AirdropError;

pub struct InitializeAirdropArgs {
    pub airdrop_amount: u64,
    pub metadata_prefix: [u8; 32],
    pub symbol: [u8; 8],
    pub price: u64,
}

pub struct InitializeAirdropUserDataArgs {}

pub enum AirdropInstruction {
    ///
    /// Accounts required:
    /// 0. `[writeable]`. Airdrop account. Used to store all of the airdrop data
    /// 1. `[]`. Airdrop authority. Account that will have the authority to airdrop nfts
    /// 2. `[]`. Mint authority. It will be used to mint airdropped nfts
    /// 3. `[]`. Revenues wallet. Wallet where all revenues are paid out
    /// 4. `[]`. Rent sysvar
    /// 5. `[signer]`. Fee payer. Wallet that will pay for creating mint authority
    InitializeAirdrop(InitializeAirdropArgs),

    ///
    /// Accounts required:
    /// 0. `[writeable]`. User data account. Used to store user data
    /// 1. `[]`. User. Wallet that will use the account
    /// 2. `[writeable]`. Airdrop. Airdrop that data account will be associated with
    /// 3. `[]`. Rent sysvar
    /// 4. `[signer]`. Fee payer. Wallet that is paying fee for creating an account
    InitializeAirdropUser(InitializeAirdropUserDataArgs),
}

fn parse_initialize_airdrop_args(body: &[u8]) -> Result<InitializeAirdropArgs, ProgramError> {
    let body_sized: &[u8; 56] = body
        .try_into()
        .or(Err(AirdropError::BadInstructionArgument))?;

    let (airdrop_amount_array, metadata_prefix_array, symbol_array, price_array) =
        array_refs!(body_sized, 8, 32, 8, 8);

    let airdrop_amount = u64::from_le_bytes(*airdrop_amount_array);
    let metadata_prefix = *metadata_prefix_array;
    let symbol = *symbol_array;
    let price = u64::from_le_bytes(*price_array);

    Ok(InitializeAirdropArgs {
        airdrop_amount,
        metadata_prefix,
        symbol,
        price,
    })
}

fn parse_initialize_airdrop_user_args(
    _body: &[u8],
) -> Result<InitializeAirdropUserDataArgs, ProgramError> {
    Ok(InitializeAirdropUserDataArgs {})
}

pub fn deserialize_instruction_data(
    instruction_data: &[u8],
) -> Result<AirdropInstruction, ProgramError> {
    let (id, body) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match id {
        1 => Ok(AirdropInstruction::InitializeAirdrop(
            parse_initialize_airdrop_args(body)?,
        )),
        2 => Ok(AirdropInstruction::InitializeAirdropUser(
            parse_initialize_airdrop_user_args(body)?,
        )),
        _ => Err(AirdropError::BadInstructionId.into()),
    }
}
