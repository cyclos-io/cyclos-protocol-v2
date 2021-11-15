use anchor_lang::prelude::*;
use anchor_spl::associated_token::{get_associated_token_address, AssociatedToken};
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::mem::size_of;

use crate::error::ErrorCode;
use crate::states::factory::FactoryState;
use crate::states::fee::FeeState;
use crate::states::pool::PoolState;
use crate::states::position::PositionState;
use crate::states::tick::TickState;
use crate::states::tick_bitmap::TickBitmapState;

// use non_fungible_position_manager::program::NonFungiblePositionManager;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    pub owner: Signer<'info>,

    #[account(
        init,
        seeds = [],
        bump = bump,
        payer = owner,
        space = size_of::<FactoryState>() + 10
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(fee: u32, tick_spacing: u16, fee_bump: u8)]
pub struct EnableFeeAmount<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [],
        bump = factory_state.bump,
        constraint = owner.key() == factory_state.owner
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        init,
        seeds = [&fee.to_be_bytes()],
        bump = fee_bump,
        payer = owner,
        space = size_of::<FeeState>() + 10
    )]
    pub fee_state: Box<Account<'info, FeeState>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(fee: u32, token_0: Pubkey, token_1: Pubkey, tick_lower:u128, tick_upper:u128, bump: u8)]
pub struct CreatePosition<'info> {
    pub owner: Signer<'info>,
    #[account(
        init, // should be mut ?
        seeds = [token_0.as_ref(), token_1.as_ref(), &fee.to_be_bytes(), &tick_lower.to_be_bytes(), &tick_upper.to_be_bytes()],
        bump = bump,
        payer = owner,
        space = size_of::<FeeState>() + 10
    )]
    pub pool_state: Box<Account<'info, PositionState>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(pool_state_bump: u8, fee: u32)]
pub struct CreatePool<'info> {
    pub pool_creator: Signer<'info>,

    #[account(
        constraint = token_0.key() != token_1.key(),
        constraint = token_0.key() < token_1.key()
    )]
    pub token_0: Box<Account<'info, Mint>>,
    pub token_1: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [token_0.key().as_ref(), token_1.key().as_ref(), &fee.to_be_bytes()],
        bump = pool_state_bump,
        payer = pool_creator,
        space = size_of::<PoolState>() + 10
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        seeds = [&fee.to_be_bytes()],
        bump = fee_state.bump,
    )]
    pub fee_state: Box<Account<'info, FeeState>>,

    // Create associated token accounts for pool_state, if not already created
    #[account(
        mut,
        address = get_associated_token_address(
            &pool_state.key(),
            &token_0.key()
        ) @ErrorCode::NotAssociatedTokenAccount
    )]
    pub vault_0: AccountInfo<'info>,

    #[account(
        mut,
        address = get_associated_token_address(
            &pool_state.key(),
            &token_1.key()
        ) @ErrorCode::NotAssociatedTokenAccount
    )]
    pub vault_1: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(position_bump: u8, tick_lower:u32, tick_upper:u32)]
pub struct MintAccount<'info> {
    pub minter: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [
            pool_state.token_0.key().as_ref(),
            pool_state.token_1.key().as_ref(),
            &pool_state.fee.to_be_bytes(),
            &tick_lower.to_be_bytes(),
            &tick_upper.to_be_bytes()
        ],
        bump = position_bump,
        payer = minter
    )]
    pub position_state: Box<Account<'info, PositionState>>,

    #[account(
        mut,
        seeds = [
            pool_state.token_0.key().as_ref(),
            pool_state.token_1.key().as_ref(),
            &pool_state.fee.to_be_bytes()
        ],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        init_if_needed,
        seeds = [
            pool_state.token_0.key().as_ref(),
            pool_state.token_1.key().as_ref(),
            &pool_state.fee.to_be_bytes(),
            &tick_lower.to_be_bytes()
        ],
        bump = position_bump,
        payer = minter
    )]
    pub tick_lower_state: Box<Account<'info, TickState>>,

    // How to save variables if it was just initialized?
    #[account(
        init_if_needed,
        seeds = [
            pool_state.token_0.key().as_ref(),
            pool_state.token_1.key().as_ref(),
            &pool_state.fee.to_be_bytes(),
            &tick_upper.to_be_bytes()
        ],
        bump = position_bump,
        payer = minter
    )]
    pub tick_upper_state: Box<Account<'info, TickState>>,
    pub tick_lower_bitmap: Box<Account<'info, TickBitmapState>>,
    pub tick_upper_bitmap: Box<Account<'info, TickBitmapState>>,

    #[account(mut)]
    pub token_account_0: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub token_account_1: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_0,
        associated_token::authority = pool_state,
    )]
    pub vault_0: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_1,
        associated_token::authority = pool_state,
    )]
    pub vault_1: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    // pub callback_handler: Program<'info, NonFungiblePositionManager>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SetOwner<'info> {
    pub owner: Signer<'info>,
    pub new_owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [],
        bump = factory_state.bump,
        constraint = owner.key() == factory_state.owner @ErrorCode::NotAnOwner
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,
}

#[derive(Accounts)]
pub struct SetFeeProtocol<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [
            pool_state.token_0.as_ref(),
            pool_state.token_1.as_ref(),
            &pool_state.fee.to_be_bytes()
        ],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        mut,
        seeds = [],
        bump = factory_state.bump,
        constraint = owner.key() == factory_state.owner @ErrorCode::NotAnOwner
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,
}

#[derive(Accounts)]
pub struct CollectProtocol<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [],
        bump = factory_state.bump,
        constraint = owner.key() == factory_state.owner @ErrorCode::NotAnOwner
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        mut,
        seeds = [
            &pool_state.fee_protocol.to_be_bytes(),
        ],
        bump = pool_state.bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_0.key(),
        associated_token::authority = pool_state,
    )]
    pub vault_0: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_1.key(),
        associated_token::authority = pool_state,
    )]
    pub vault_1: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_0.key(),
        associated_token::authority = owner.key(),
    )]
    pub owner_wallet_0: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_state.token_1.key(),
        associated_token::authority = owner.key(),
    )]
    pub owner_wallet_1: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
