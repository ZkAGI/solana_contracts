use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{self, mint_to, Mint, MintTo, Token, TokenAccount, Transfer as SplTransfer},
};

mod constants;
mod errors;
mod events;
mod utils;

declare_id!("FtxVHduAJmYGWTFRAWpUvtpkphK42EBvqezQ9HGcMHyp");

#[program]
pub mod ZkAGI_Contracts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, init_params: PlatformInitParams) -> Result<()> {
        require!(
            init_params.fee_in_bps <= constants::general::MAX_ALLOWED_FEE_IN_BPS,
            errors::CustomErrors::ExcessiveFees
        );

        let platform = &mut ctx.accounts.platform;

        platform.fee_in_bps = init_params.fee_in_bps;
        platform.owner = init_params.owner;
        platform.total_supply = init_params.total_supply;
        platform.virtual_sol = init_params.virtual_sol;
        platform.target_pool_balance = init_params.target_pool_balance;

        let platform_initialized_event = events::PlatformInitialized {
            platform: ctx.accounts.platform.key(),
            owner: ctx.accounts.signer.key(),
        };
        emit!(platform_initialized_event);

        Ok(())
    }

    pub fn change_owner(ctx: Context<PlatformOperation>, new_owner: Pubkey) -> Result<()> {
        ctx.accounts.platform.owner = new_owner;

        let owner_changed_event = events::OwnerChanged { new_owner };
        emit!(owner_changed_event);

        Ok(())
    }

    pub fn change_fees(ctx: Context<PlatformOperation>, new_fees: u64) -> Result<()> {
        ctx.accounts.platform.fee_in_bps = new_fees;

        let fees_changed_event = events::FeesChanged { new_fees };
        emit!(fees_changed_event);

        Ok(())
    }

    pub fn change_total_supply(
        ctx: Context<PlatformOperation>,
        new_total_supply: u64,
    ) -> Result<()> {
        ctx.accounts.platform.total_supply = new_total_supply;

        let total_supply_changed_event = events::TotalSupplyChanged { new_total_supply };
        emit!(total_supply_changed_event);

        Ok(())
    }

    pub fn change_virtual_sol_amount(
        ctx: Context<PlatformOperation>,
        new_virtual_sol_amount: u64,
    ) -> Result<()> {
        ctx.accounts.platform.virtual_sol = new_virtual_sol_amount;

        let virtual_sol_amount_changed_event = events::VirtualSolChanged {
            new_virtual_sol_amount,
        };
        emit!(virtual_sol_amount_changed_event);

        Ok(())
    }

    pub fn change_target_pool_balance(
        ctx: Context<PlatformOperation>,
        new_target_pool_balance: u64,
    ) -> Result<()> {
        ctx.accounts.platform.target_pool_balance = new_target_pool_balance;

        let target_pool_balance_changed_event = events::TargetPoolBalanceChanged {
            new_target_pool_balance,
        };
        emit!(target_pool_balance_changed_event);

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        let platform = &mut ctx.accounts.platform;
        let accumulated_fees = platform.accumulated_fees;

        **platform.to_account_info().try_borrow_mut_lamports()? -= accumulated_fees;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += accumulated_fees;

        platform.accumulated_fees = 0;

        let fees_withdrawn_event = events::FeesWithdrawn {
            amount: accumulated_fees,
        };
        emit!(fees_withdrawn_event);

        Ok(())
    }

    pub fn create_token(
        ctx: Context<CreateToken>,
        create_token_params: CreateTokenParams,
    ) -> Result<()> {
        let seeds = &[
            constants::seeds::MINT_SEED,
            create_token_params.name.as_bytes(),
            &[ctx.bumps.mint],
        ];
        let signer = [&seeds[..]];

        // First, create the token metadata
        // The token was already created by Anchor in the background
        let token_data: DataV2 = DataV2 {
            name: create_token_params.name.clone(),
            symbol: create_token_params.symbol,
            uri: create_token_params.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );
        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

        // Second, initialize the token campaign params
        let token_info = &mut ctx.accounts.token_info;
        token_info.token = ctx.accounts.mint.key();
        token_info.total_supply = ctx.accounts.platform.total_supply;
        token_info.virtual_sol = ctx.accounts.platform.virtual_sol;
        token_info.sol_reserve = token_info.virtual_sol;
        token_info.token_reserve = token_info.total_supply;
        token_info.target_pool_balance = ctx.accounts.platform.target_pool_balance;

        // Lastly, mint the total supply of the tokens to the token vault associated with this token launch
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            ctx.accounts.token_info.total_supply,
        )?;

        let token_created_event = events::TokenCreated {
            token: ctx.accounts.mint.key(),
        };
        emit!(token_created_event);

        Ok(())
    }

    pub fn init_token_account_for_token(_: Context<InitAccountsForToken>) -> Result<()> {
        msg!("Token account created");

        Ok(())
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
        require!(
            !ctx.accounts.token_info.launched,
            errors::CustomErrors::AlreadyLaunched
        );

        // sol_amount = 101
        // buy_fee = (101 * 100) / (10000 + 100) = 1
        // sol_amount_after_fee = 100
        let fee_amount = utils::calculate_buy_fee(
            &(sol_amount as u128),
            &(ctx.accounts.platform.fee_in_bps as u128),
        );
        let sol_amount_after_fee = sol_amount - fee_amount;
        let token_amount = utils::get_amount_out(
            &(sol_amount_after_fee as u128),
            &(ctx.accounts.token_info.sol_reserve as u128),
            &(ctx.accounts.token_info.token_reserve as u128),
        );

        ctx.accounts.platform.accumulated_fees += fee_amount;

        ctx.accounts.token_info.sol_reserve += sol_amount_after_fee;
        ctx.accounts.token_info.token_reserve -= token_amount;

        require!(
            ctx.accounts.token_info.sol_reserve <= ctx.accounts.token_info.target_pool_balance,
            errors::CustomErrors::BondingCurveBreached
        );

        // Check the current market cap and launch the token if it's been hit
        if ctx.accounts.token_info.sol_reserve >= ctx.accounts.token_info.target_pool_balance {
            ctx.accounts.token_info.launched = true;
        }

        // Transfer sol amount (after applying fee) from signer to token info account
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.token_info.to_account_info(),
                },
            ),
            sol_amount_after_fee,
        )?;

        // Transfer fees (in sol) to the platform account
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.platform.to_account_info(),
                },
            ),
            fee_amount,
        )?;

        let mint_token_account_key = ctx.accounts.mint.key();
        let seeds = &[
            constants::seeds::TOKEN_ACCOUNT_SEED,
            mint_token_account_key.as_ref(),
            &[ctx.bumps.source_token_account],
        ];
        let signer = [&seeds[..]];

        // Transfer tokens to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.source_token_account.to_account_info().clone(),
                    to: ctx.accounts.user_token_account.to_account_info().clone(),
                    authority: ctx.accounts.source_token_account.to_account_info().clone(),
                },
                &signer,
            ),
            token_amount,
        )?;

        Ok(())
    }

    pub fn sell_tokens(ctx: Context<SellTokens>, token_amount: u64, name: String) -> Result<()> {
        require!(
            !ctx.accounts.token_info.launched,
            errors::CustomErrors::AlreadyLaunched
        );

        let sol_amount = utils::get_amount_out(
            &(token_amount as u128),
            &(ctx.accounts.token_info.token_reserve as u128),
            &(ctx.accounts.token_info.sol_reserve as u128),
        );
        let fee_amount = utils::calculate_sell_fee(
            &(sol_amount as u128),
            &(ctx.accounts.platform.fee_in_bps as u128),
        );
        let sol_amount_after_fee = sol_amount - fee_amount;

        ctx.accounts.token_info.sol_reserve -= sol_amount;
        ctx.accounts.token_info.token_reserve += token_amount;

        ctx.accounts.platform.accumulated_fees += fee_amount;

        // Transfer tokens from user to token account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.source_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            token_amount,
        )?;

        **ctx
            .accounts
            .token_info
            .to_account_info()
            .try_borrow_mut_lamports()? -= sol_amount_after_fee;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += sol_amount_after_fee;

        **ctx
            .accounts
            .token_info
            .to_account_info()
            .try_borrow_mut_lamports()? -= fee_amount;
        **ctx
            .accounts
            .platform
            .to_account_info()
            .try_borrow_mut_lamports()? += fee_amount;

        let tokens_sold_event = events::TokensSold {
            token: ctx.accounts.mint.key(),
            by: ctx.accounts.signer.key(),
            amount: token_amount,
        };
        emit!(tokens_sold_event);

        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, name: String) -> Result<()> {
        require!(
            ctx.accounts.token_info.launched,
            errors::CustomErrors::NotLaunched
        );

        let sol_amount = ctx.accounts.token_info.sol_reserve - ctx.accounts.token_info.virtual_sol;
        let token_amount = utils::get_amount_using_spot_price(
            &(sol_amount as u128),
            &(ctx.accounts.token_info.sol_reserve as u128),
            &(ctx.accounts.token_info.token_reserve as u128),
        );

        let mint_token_account_key = ctx.accounts.mint.key();
        let token_transfer_seeds = &[
            constants::seeds::TOKEN_ACCOUNT_SEED,
            mint_token_account_key.as_ref(),
            &[ctx.bumps.source_token_account],
        ];
        let token_transfer_signer = [&token_transfer_seeds[..]];

        // Transfer token amount to admin
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SplTransfer {
                    from: ctx.accounts.source_token_account.to_account_info().clone(),
                    to: ctx.accounts.user_token_account.to_account_info().clone(),
                    authority: ctx.accounts.source_token_account.to_account_info().clone(),
                },
                &token_transfer_signer,
            ),
            token_amount,
        )?;

        **ctx
            .accounts
            .token_info
            .to_account_info()
            .try_borrow_mut_lamports()? -= sol_amount;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += sol_amount;

        let liquidity_added_event = events::LiquidityAdded {
            token: ctx.accounts.mint.key(),
            sol_amount,
            token_amount,
        };
        emit!(liquidity_added_event);

        Ok(())
    }
}

// Contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=signer, seeds=[constants::seeds::PLATFORM_SEED], bump, space=constants::general::DISCRIMINATOR_SIZE + Platform::INIT_SPACE)]
    pub platform: Account<'info, Platform>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlatformOperation<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump, constraint=platform.owner == signer.key() @ errors::CustomErrors::NotOwner)]
    pub platform: Account<'info, Platform>,
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump, constraint = platform.owner == signer.key())]
    pub platform: Account<'info, Platform>,
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(create_token_params: CreateTokenParams)]
pub struct CreateToken<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump)]
    pub platform: Box<Account<'info, Platform>>,
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(init, payer=signer, seeds=[constants::seeds::MINT_SEED, create_token_params.name.as_bytes()], bump, mint::decimals=constants::general::DECIMALS, mint::authority=mint)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(init, payer=signer, seeds=[constants::seeds::TOKEN_ACCOUNT_SEED, mint.key().as_ref()], bump, token::mint=mint, token::authority=token_account)]
    pub token_account: Box<Account<'info, TokenAccount>>,
    #[account(init, payer=signer, seeds=[constants::seeds::TOKEN_SEED, create_token_params.name.as_ref()], bump, space=constants::general::DISCRIMINATOR_SIZE + TokenInfo::INIT_SPACE,)]
    pub token_info: Box<Account<'info, TokenInfo>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitAccountsForToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account()]
    pub mint: Account<'info, Mint>,
    #[account(init, payer=signer, seeds=[constants::seeds::TOKEN_ACCOUNT_SEED, signer.key().as_ref(), mint.key().as_ref()], bump,  token::mint=mint, token::authority=signer)]
    pub token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump)]
    pub platform: Account<'info, Platform>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account()]
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint=mint.key() == token_info.token)]
    pub token_info: Account<'info, TokenInfo>,
    #[account(mut, seeds=[constants::seeds::TOKEN_ACCOUNT_SEED, mint.key().as_ref()], bump, token::mint=mint, token::authority=source_token_account)]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(token_amount: u64, name: String)]
pub struct SellTokens<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump)]
    pub platform: Account<'info, Platform>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, seeds=[constants::seeds::TOKEN_SEED, name.as_bytes()], bump, constraint=mint.key() == token_info.token)]
    pub token_info: Box<Account<'info, TokenInfo>>,
    #[account(mut, seeds=[constants::seeds::TOKEN_ACCOUNT_SEED, mint.key().as_ref()], bump, token::mint=mint, token::authority=source_token_account)]
    pub source_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddLiquidity<'info> {
    #[account(mut, seeds=[constants::seeds::PLATFORM_SEED], bump)]
    pub platform: Account<'info, Platform>,
    #[account(mut, constraint=signer.key() == platform.owner)]
    pub signer: Signer<'info>,

    #[account()]
    pub mint: Account<'info, Mint>,
    #[account(mut, seeds=[constants::seeds::TOKEN_SEED, name.as_ref()], bump, constraint=mint.key() == token_info.token)]
    pub token_info: Account<'info, TokenInfo>,
    #[account(mut, seeds=[constants::seeds::TOKEN_ACCOUNT_SEED, mint.key().as_ref()], bump,  token::mint=mint, token::authority=source_token_account)]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

// Accounts

#[account]
#[derive(InitSpace)]
pub struct Platform {
    pub owner: Pubkey,
    pub fee_in_bps: u64,
    pub accumulated_fees: u64,
    pub total_supply: u64,
    pub virtual_sol: u64,
    pub target_pool_balance: u64,
}

#[account]
#[derive(InitSpace)]
pub struct TokenInfo {
    pub token: Pubkey,
    pub creator: Pubkey,
    pub total_supply: u64,
    pub virtual_sol: u64,
    pub sol_reserve: u64,
    pub token_reserve: u64,
    pub target_pool_balance: u64,
    pub launched: bool,
}

// Params

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct PlatformInitParams {
    pub owner: Pubkey,
    pub fee_in_bps: u64,
    pub total_supply: u64,
    pub virtual_sol: u64,
    pub target_pool_balance: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct CreateTokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}
