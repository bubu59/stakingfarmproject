use anchor_lang::prelude::*;

use crate::state::*;
use anchor_spl::token::{self, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

#[derive(Accounts)]
pub struct CreateGame<'info> {
  // define accounts taken in by the CreateGame instruction
  #[account(init, payer = host, space = 8 + (32 * 4) + 16 + (8 * 6) + (1 * 3))]
  pub game_config: Account<'info, Game>,

  #[account(mut)]
  pub host: Signer<'info>,

  #[account(mut, constraint = host_reward_account.mint == reward_mint.key())]
  pub host_reward_account: Account<'info, TokenAccount>,

  #[account(mut)]
  pub reward_mint: Account<'info, Mint>,

  #[account(
      init,
      seeds = [b"reward-escrow".as_ref(), game_config.to_account_info().key.as_ref()],
      bump,
      payer = host,
      token::mint = reward_mint,
      token::authority = host,
  )]
  pub reward_escrow: Account<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  pub token_program: AccountInfo<'info>,
}