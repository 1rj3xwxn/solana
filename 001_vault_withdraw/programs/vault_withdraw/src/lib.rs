pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

use crate::error::ErrorCode;

declare_id!("5owRe6K1Yhn2f34HSiR7zh54Dne6xqwzjaKz9tr2EtKp");

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Pay<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"nest", user.key().as_ref()],
        bump
    )]
    pub nest: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendOut<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub recipient: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"nest", user.key().as_ref()],
        bump
    )]
    pub nest: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[program]
pub mod vault_withdraw {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx)
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        crate::instructions::increment::handle_increment(ctx)
    }

    /// 从该用户的金库转出 1_000_000 lamports。
    ///
    /// `ctx` 里的 `user` 是收款人，也是种子中的公钥；`vault` 是程序代持的系统账户。
    /// 成功时返回 `Ok(())`。
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let user_key = ctx.accounts.user.key();
        let bump = [ctx.bumps.vault];
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", user_key.as_ref(), bump.as_ref()]];

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            cpi_accounts,
            signer_seeds,
        );
        anchor_lang::system_program::transfer(cpi_ctx, 1_000_000)?;
        Ok(())
    }

    pub fn pay(ctx: Context<Pay>) -> Result<()> {
        let user_key = ctx.accounts.user.key();
        let bump = [ctx.bumps.nest];
        let signer_seeds: &[&[&[u8]]] = &[&[b"nest", user_key.as_ref(), bump.as_ref()]];

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.nest.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            cpi_accounts,
            signer_seeds,
        );
        anchor_lang::system_program::transfer(cpi_ctx, 500_000)?;
        Ok(())
    }

     pub fn send_out(ctx: Context<SendOut>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::ZeroAmount);
        let user_key = ctx.accounts.user.key();
        let bump = [ctx.bumps.nest];
        let signer_seeds: &[&[&[u8]]] = &[&[b"nest", user_key.as_ref(), bump.as_ref()]];

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: ctx.accounts.nest.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            anchor_lang::system_program::ID,
            cpi_accounts,
            signer_seeds,
        );
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
