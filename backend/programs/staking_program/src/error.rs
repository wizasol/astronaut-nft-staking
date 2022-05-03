use anchor_lang::prelude::*;

#[error]
pub enum StakingError {
    #[msg("Invalid User Pool")]
    InvalidUserPool,
    #[msg("Invalid Collection")]
    InvalidCollection,
    #[msg("Invalid User Pool")]
    InvalidAdmin,
    #[msg("Invalid pool number")]
    InvalidPoolError,
    #[msg("No Matching NFT to withdraw")]
    InvalidNFTAddress,
    #[msg("NFT Owner key mismatch")]
    InvalidOwner,
    #[msg("Staking Locked Now")]
    InvalidWithdrawTime,
    #[msg("Withdraw NFT Index OverFlow")]
    IndexOverflow,
    #[msg("You can't Unstake Before LockTime")]
    BeforeLockTime,
    #[msg("Insufficient Lamports")]
    LackLamports,
    #[msg("Can't Parse The NFT's Creators")]
    MetadataCreatorParseError,
    #[msg("Invalid Metadata Address")]
    InvaliedMetadata,
}
