use anchor_lang::prelude::*;

use crate::state::*;
use anchor_spl::token::{self, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Stake<'info> {
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

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    if amount == 0 {
        return Err(error!(ErrorCode::AmountMustBeGreaterThanZero));
    }
    let _pool = &mut ctx.accounts.pool;
    //updating total staked
    let _total_staked = &mut ctx
        .accounts
        .staking_vault
        .amount
        .checked_add(amount)
        .unwrap();

    //to keep things simple, for now the ratio will be 2:1.. so amt of reward token = amount staked / 2
    let reward_amount: u64 = (amount as u128).checked_div(2).unwrap().try_into().unwrap();

    //update reward balance of user
    ctx.accounts.user.reward_balance = ctx
        .accounts
        .user
        .reward_balance
        .checked_add(reward_amount)
        .unwrap();

    //transfer reward token from reward token vault into user reward token account
    let seed = &[
        ctx.accounts.pool.to_account_info().key.as_ref(),
        &[ctx.accounts.pool.nonce],
    ];
    let pool_signer = &[&seed[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.reward_token_vault.to_account_info(),
            to: ctx.accounts.user_reward_token_account.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        },
        pool_signer,
    );
    token::transfer(cpi_ctx, reward_amount)?;

    //update the amount of balance staked on the user account
    ctx.accounts.user.balance_staked = ctx
        .accounts
        .user
        .balance_staked
        .checked_add(amount)
        .unwrap();
    //Here, we transfer the tokens into staking vault
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.user_staking_token_account.to_account_info(),
            to: ctx.accounts.staking_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
