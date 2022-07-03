use solana_program::pubkey::Pubkey;

use crate::state::{MINT_AUTHORITY, USER_DATA};

pub fn find_airdrop_user_data(airdrop_config: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[USER_DATA.as_bytes(), airdrop_config.as_ref(), user.as_ref()],
        &crate::id(),
    )
}

pub fn find_mint_authority(airdrop_config: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[MINT_AUTHORITY.as_bytes(), airdrop_config.as_ref()],
        &crate::id(),
    )
}
