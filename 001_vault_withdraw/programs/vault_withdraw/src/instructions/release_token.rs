use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::POT_SEED;
use crate::error::ErrorCode;

/// 把 PDA 金库里的代币转给用户自己的代币账户。
///
/// `pot` 是 authority，不存放代币。`pot_token` 才存放代币，它的 authority 是 `pot`。
/// `user_token` 的 authority 是 `user`，转出的代币进这里。
#[derive(Accounts)]
pub struct ReleaseToken<'info> {
    pub user: Signer<'info>,
    /// CHECK: 只用来签名。种子约束保证它是这个 `user` 的 pot。
    #[account(
        seeds = [POT_SEED, user.key().as_ref()],
        bump
    )]
    pub pot: UncheckedAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = pot,
    )]
    pub pot_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// 用 `pot` 的种子签名，把 `amount` 个代币从 `pot_token` 转到 `user_token`。
///
/// `ctx` 是本指令的账户。`amount` 必须大于 0。
/// 成功时返回 `Ok(())`。金额为 0 时返回 `ZeroAmount`，不调用 Token 程序。
pub fn handle_release_token(ctx: Context<ReleaseToken>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroAmount);

    let user_key = ctx.accounts.user.key();
    let bump = [ctx.bumps.pot];
    let signer_seeds: &[&[&[u8]]] = &[&[POT_SEED, user_key.as_ref(), bump.as_ref()]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.pot_token.to_account_info(),
        to: ctx.accounts.user_token.to_account_info(),
        authority: ctx.accounts.pot.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        token::ID,
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
