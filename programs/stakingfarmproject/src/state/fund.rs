use anchor_lang::prelude::*;

#[account]
pub struct Fund {
    //The owner of this account
    pub staker: Pubkey,
    //reward balance
    pub reward_balance: u64,
    //The amount staked
    pub balance_staked: u64,
    //Signer nonce(for PDA)
    pub fund_bump: u8,
}

impl Fund {
    //leave empty
}