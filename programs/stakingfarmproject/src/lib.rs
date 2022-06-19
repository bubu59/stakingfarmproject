use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
// use spl_token::instruction::AuthorityType;
use std::convert::TryInto;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod simplestaking {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, pool_nonce: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.nonce = pool_nonce;
        pool.staking_vault = ctx.accounts.staking_vault.key();
        pool.staking_mint = ctx.accounts.staking_mint.key();
        pool.reward_token_mint = ctx.accounts.reward_token_mint.key();
        pool.reward_token_vault = ctx.accounts.reward_token_vault.key();
        pool.user_stake_count = 0;
        Ok(())
    }

    //Create a new user account (initialization)
    pub fn create_user(ctx: Context<CreateUser>, nonce: u8) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.nonce = nonce;
        user.pool = ctx.accounts.pool.key();
        user.owner = ctx.accounts.owner.key();
        user.balance_staked = 0;
        user.reward_balance = 0;
        //update user count in pool
        let pool = &mut ctx.accounts.pool;
        pool.user_stake_count = pool.user_stake_count.checked_add(1).unwrap();
        Ok(())
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
}

#[derive(Accounts)]
#[instruction(pool_nonce: u8)]
pub struct InitializePool<'info> {
    //admin wallet
    authority: UncheckedAccount<'info>,
    //staking token mint
    staking_mint: Box<Account<'info, Mint>>,
    //staking token vault
    #[account(constraint = staking_vault.mint == staking_mint.key(), constraint = staking_vault.owner == pool_signer.key(),)]
    staking_vault: Box<Account<'info, TokenAccount>>,
    //reward token mint
    reward_token_mint: Box<Account<'info, Mint>>,
    //reward token vault
    #[account(constraint = reward_token_vault.mint == reward_token_mint.key(), constraint = reward_token_vault.owner == pool_signer.key(),)]
    reward_token_vault: Box<Account<'info, TokenAccount>>,
    //pool signer
    #[account(seeds = [pool.to_account_info().key.as_ref()], bump = pool_nonce,)]
    pool_signer: UncheckedAccount<'info>,
    //pool
    #[account(zero)]
    pool: Box<Account<'info, Pool>>,
    //token program
    token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct CreateUser<'info> {
    #[account(mut)]
    pool: Box<Account<'info, Pool>>,
    #[account(init, payer = owner, seeds = [owner.key.as_ref(), pool.to_account_info().key.as_ref()], bump,)]
    user: Box<Account<'info, User>>,
    #[account(mut)]
    owner: Signer<'info>,
    system_program: Program<'info, System>,
}

#[account]
pub struct Pool {
    //Priveleged account
    pub authority: Pubkey,
    //Nonce to derive PDA owning the vaults
    pub nonce: u8,
    //Mint of token that can be staked
    pub staking_mint: Pubkey,
    //Vault to store staked tokens
    pub staking_vault: Pubkey,
    //reward token mint
    pub reward_token_mint: Pubkey,
    //reward token vault
    pub reward_token_vault: Pubkey,
    //no of stakers
    pub user_stake_count: u32,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero!")]
    AmountMustBeGreaterThanZero,
    #[msg("Amount must be less than balance staked!")]
    InsufficientBalance,
}


