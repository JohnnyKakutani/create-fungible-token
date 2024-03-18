use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_lang::solana_program::program_error::ProgramError;

declare_id!("7HP7zAngQSTJR6nv7wFjDzeWTcRLSX9qdEdok5e1YZnC");

#[program]
pub mod solana_staking_blog {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Instruction Initialize");

        let pool_info = &mut ctx.accounts.pool_info;

        pool_info.admin = ctx.accounts.admin.key();

        pool_info.token = ctx.accounts.staking_token.key();

        pool_info.amount = 0;

        Ok(())

    }

    pub fn stake(ctx: Context<Stake>, amount: u64, lockedays: u64) -> Result<()> {
        msg!("Instruction Stake");

        let user_info = &mut ctx.accounts.user_info;
        let pool_info = &mut ctx.accounts.pool_info; 
        let clock = Clock::get()?;

        if clock.slot - user_info.deposit_slot < user_info.locked_days {
            Err(())
        }

        if user_info.reward > 0 {
            let reward = user_info.reward;
            let cpi_accounts = MintTo {
                mint: ctx.accounts.staking_token.to_account_info(),
                to: ctx.accounts.user_staking_wallet.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

            token::mint_to(cpi_ctx, reward)?;
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_staking_wallet.to_account_info(),
            to: ctx.accounts.admin_staking_wallet.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        user_info.amount += amount;
        user_info.deposit_slot = clock.slot;
        user_info.locked_days = lockedays;
        user_info.reward = user_info.amount * user_info.locked_days / 10;
        pool_info.amount += amount;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        msg!("Instruction Unstake");

        let user_info = &mut ctx.accounts.user_info;
        let clock = Clock::get()?;

        if clock.slot - user_info.deposit_slot < user_info.locked_days {
            Err(())
        }

        let reward = user_info.reward;

        let cpi_accounts = MintTo {
            mint: ctx.accounts.staking_token.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, reward)?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.admin_staking_wallet.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        user_info.amount -= amount;
        pool_info.amount -= amount;
        user_info.deposit_slot = 0;
        user_info.locked_days = 0;
        user_info.reward = 0;
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {

        msg!("Instruction Claim Reward");

        let user_info = &mut ctx.accounts.user_info;

        let clock = Clock::get()?;

        if clock.slot - user_info.deposit_slot < user_info.locked_days {
            Err(())
        }

        let reward = user_info.reward;

        if reward > 0 {
            let cpi_accounts = MintTo {
                mint: ctx.accounts.staking_token.to_account_info(),
                to: ctx.accounts.user_staking_wallet.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::mint_to(cpi_ctx, reward)?;
            user_info.reward = 0;
            Ok(())
        }

        Err(())
    }

    pub fn get_userinfo(ctx: Context<UserDatas>) -> Result<UserInfo, ProgramError> {
        Ok(*ctx.accounts.user_info);
    }

    pub fn get_poolinfo(ctx: Context<PoolDatas>) -> Result<u64, ProgramError> {
        Ok(*ctx.accounts.pool_info.amount);
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(init, payer = admin, space = 8 + PoolInfo::LEN)]
    pub pool_info: Account<'info, PoolInfo>,
    
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    
    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]

pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(init, payer = user, space = 8 + UserInfo::LEN)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub pool_info: Account<'info, PoolInfo>,

    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]

pub struct Unstake<'info> {
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub pool_info: Account<'info, PoolInfo>,

    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,

}

#[derive(Accounts)]
pub struct ClaimReward<'info> {

    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub admin: AccountInfo<'info>,

    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct UserDatas<'info> {
    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,
}

#[derive(Accounts)]
pub struct PoolDatas<'info> {
    #[account(mut)]
    pub pool_info: Account<'info, PoolInfo>,
}

#[account]
pub struct PoolInfo {
    pub admin: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}

#[account]

pub struct UserInfo {
    pub amount: u64,
    pub deposit_slot: u64,
    pub locked_days: u64,
    pub reward: u64,
}

impl UserInfo {
    pub const LEN: usize = 8 + 8 + 8 + 8;
}

impl PoolInfo {
    pub const LEN: usize = 32 + 32 + 8;
}