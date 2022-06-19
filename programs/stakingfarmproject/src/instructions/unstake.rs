use anchor_lang::prelude::*;

use crate::state::*;
use anchor_spl::token::{self, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct UnStake<'info> {
    //Pool
    #[account(mut, has_one = staking_vault,)]
    pool: Box<Account<'info, Pool>>,
    //Staking vault
    #[account(mut, constraint = staking_vault.owner == *pool_signer.key,)]
    staking_vault: Box<Account<'info, TokenAccount>>,
    //User
    #[account(mut, has_one = owner, has_one = pool, seeds = [owner.key.as_ref(), pool.to_account_info().key.as_ref()], bump = user.nonce,)]
    user: Box<Account<'info, User>>,
    owner: Signer<'info>,
    //user staking token account
    #[account(mut)]
    user_staking_token_account: Box<Account<'info, TokenAccount>>,
    //user reward token account
    #[account(mut)]
    user_reward_token_account: Box<Account<'info, TokenAccount>>,
    //reward token mint
    reward_token_mint: Box<Account<'info, Mint>>,
    //reward token vault
    #[account(constraint = reward_token_vault.mint == reward_token_mint.key(), constraint = reward_token_vault.owner == pool_signer.key(),)]
    reward_token_vault: Box<Account<'info, TokenAccount>>,
    //Program signer
    #[account(seeds = [pool.to_account_info().key.as_ref()], bump = pool.nonce,)]
    pool_signer: UncheckedAccount<'info>,
    //Token Program
    token_program: Program<'info, Token>,
}



pub fn unstake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    //check amount greater than 0..
    if amount == 0 {
        return Err(error!(ErrorCode::AmountMustBeGreaterThanZero));
    }
    //check if amount specified is less than staking balance
    if ctx.accounts.user.balance_staked < amount {
        return Err(error!(ErrorCode::InsufficientBalance));
    }
    let _pool = &mut ctx.accounts.pool;
    //update staking vault amount
    let _total_staked = &mut ctx
        .accounts
        .staking_vault
        .amount
        .checked_sub(amount)
        .unwrap();
    //update user staking balance
    ctx.accounts.user.balance_staked = ctx
        .accounts
        .user
        .balance_staked
        .checked_sub(amount)
        .unwrap();
    //Transfer token from staking vault to user vault
    let seeds = &[
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.nonce],
    ];
    let pool_signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.user_staking_token_account.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        },
        pool_signer,
    );
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}

pub fn handler(ctx: Context<Unstake>)