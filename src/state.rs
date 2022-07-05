use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const USER_DATA: &str = "user_data";
pub const MINT_AUTHORITY: &str = "mint_authority";

#[derive(Debug, Copy, Clone)]
pub struct AirdropConfig {
    pub initialized: bool,
    pub airdrop_authority: Pubkey,
    pub airdrop_index: u64,
    pub airdrop_amount: u64,
    pub metadata_prefix: [u8; 32],
    pub symbol: [u8; 8],
    pub airdrop_users: u64,
    pub revenues_wallet: Pubkey,
    pub admin_account: Pubkey,
    pub price: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct AirdropUserData {
    pub initialized: bool,
    pub airdrop: Pubkey,
    pub user: Pubkey,
    pub mints_amount: u64,
    pub locked_till: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct MintAuthority {}

impl Sealed for AirdropConfig {}

impl IsInitialized for AirdropConfig {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Pack for AirdropConfig {
    const LEN: usize = 1 + 32 + 8 + 8 + 32 + 8 + 8 + 32 + 32 + 8;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AirdropConfig::LEN];

        let (
            initialized,
            airdrop_authority,
            airdrop_index,
            airdrop_amount,
            metadata_prefix,
            symbol,
            airdrop_users,
            revenues_wallet,
            admin_account,
            price,
        ) = mut_array_refs![dst, 1, 32, 8, 8, 32, 8, 8, 32, 32, 8];

        initialized[0] = self.initialized as u8;
        airdrop_authority.copy_from_slice(&self.airdrop_authority.to_bytes());
        airdrop_index.copy_from_slice(&self.airdrop_index.to_le_bytes());
        airdrop_amount.copy_from_slice(&self.airdrop_amount.to_le_bytes());
        metadata_prefix.copy_from_slice(&self.metadata_prefix);
        symbol.copy_from_slice(&self.symbol);
        airdrop_users.copy_from_slice(&self.airdrop_users.to_le_bytes());
        revenues_wallet.copy_from_slice(&self.revenues_wallet.to_bytes());
        admin_account.copy_from_slice(&self.admin_account.to_bytes());
        price.copy_from_slice(&self.price.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        let src = array_ref![src, 0, AirdropConfig::LEN];

        let (
            initialized_src,
            airdrop_authority_src,
            airdrop_index_src,
            airdrop_amount_src,
            metadata_prefix_src,
            symbol_src,
            airdrop_users_src,
            revenues_wallet_src,
            admin_account_src,
            price_src,
        ) = array_refs![src, 1, 32, 8, 8, 32, 8, 8, 32, 32, 8];

        let initialized = match initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let airdrop_authority = Pubkey::new_from_array(*airdrop_authority_src);
        let airdrop_index = u64::from_le_bytes(*airdrop_index_src);
        let airdrop_amount = u64::from_le_bytes(*airdrop_amount_src);
        let symbol = symbol_src.clone();
        let metadata_prefix = metadata_prefix_src.clone();
        let airdrop_users = u64::from_le_bytes(*airdrop_users_src);
        let revenues_wallet = Pubkey::new_from_array(*revenues_wallet_src);
        let admin_account = Pubkey::new_from_array(*admin_account_src);
        let price = u64::from_le_bytes(*price_src);

        Ok(AirdropConfig {
            initialized,
            airdrop_authority,
            airdrop_index,
            airdrop_amount,
            symbol,
            metadata_prefix,
            airdrop_users,
            revenues_wallet,
            admin_account,
            price,
        })
    }
}

impl AirdropConfig {
    pub fn unpack_from_account(account: &AccountInfo) -> Result<AirdropConfig, ProgramError> {
        Self::unpack_unchecked(&account.data.borrow())
    }

    pub fn pack_into_account(
        state: AirdropConfig,
        account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::pack(state, &mut account.data.borrow_mut())
    }
}

impl Sealed for AirdropUserData {}

impl IsInitialized for AirdropUserData {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Pack for AirdropUserData {
    const LEN: usize = 1 + 32 + 32 + 8 + 8;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AirdropUserData::LEN];

        let (initialized, airdrop, user, mints_amount, locked_till) =
            mut_array_refs![dst, 1, 32, 32, 8, 8];

        initialized[0] = self.initialized as u8;
        airdrop.copy_from_slice(&self.airdrop.to_bytes());
        user.copy_from_slice(&self.user.to_bytes());
        mints_amount.copy_from_slice(&self.mints_amount.to_le_bytes());
        locked_till.copy_from_slice(&self.locked_till.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AirdropUserData::LEN];

        let (initialized_src, airdrop_src, user_src, mints_amount_src, locked_till_src) =
            array_refs![src, 1, 32, 32, 8, 8];

        let initialized = match initialized_src {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        let airdrop = Pubkey::new_from_array(*airdrop_src);
        let user = Pubkey::new_from_array(*user_src);
        let mints_amount = u64::from_le_bytes(*mints_amount_src);
        let locked_till = u64::from_le_bytes(*locked_till_src);

        Ok(AirdropUserData {
            initialized,
            airdrop,
            user,
            mints_amount,
            locked_till,
        })
    }
}

impl AirdropUserData {
    pub fn unpack_from_account(account: &AccountInfo) -> Result<AirdropUserData, ProgramError> {
        Self::unpack_unchecked(&account.data.borrow())
    }

    pub fn pack_into_account(
        state: AirdropUserData,
        account: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::pack(state, &mut account.data.borrow_mut())
    }
}

impl MintAuthority {
    pub const LEN: usize = 0;
}
