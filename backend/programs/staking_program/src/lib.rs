use anchor_lang::{accounts::cpi_account::CpiAccount, prelude::*};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount, Transfer},
};
use metaplex_token_metadata::state::Metadata;

pub mod account;
pub mod constants;
pub mod error;

use account::*;
use constants::*;
use error::*;

declare_id!("51dLXLR41vAQV5gQBExTNdDL5CadqcTh2HFvjnHoNHUz");

#[program]
pub mod staking_program {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, global_bump: u8) -> ProgramResult {
        let global_authority = &mut ctx.accounts.global_authority;
        global_authority.admin = ctx.accounts.admin.key();
        global_authority.adventure_rate = ADV_REWARD_AMOUNT;
        global_authority.scientist_rate = SCI_REWARD_AMOUNT;
        global_authority.doctor_rate = DOC_REWARD_AMOUNT;
        global_authority.specialist_rate = SPE_REWARD_AMOUNT;
        global_authority.commander_rate = COM_REWARD_AMOUNT;
        global_authority.normal_rate = NORMAL_REWARD_AMOUNT;
        Ok(())
    }

    pub fn set_amount(
        ctx: Context<SetAmount>,
        global_bump: u8,
        adv_amount: u64,
        sci_amount: u64,
        doc_amount: u64,
        spe_amount: u64,
        com_amount: u64,
        nor_amount: u64,
    ) -> ProgramResult {
        let global_authority = &mut ctx.accounts.global_authority;
        require!(
            ctx.accounts.admin.key() == global_authority.admin,
            StakingError::InvalidAdmin
        );
        global_authority.adventure_rate = adv_amount;
        global_authority.scientist_rate = sci_amount;
        global_authority.doctor_rate = doc_amount;
        global_authority.specialist_rate = spe_amount;
        global_authority.commander_rate = com_amount;
        global_authority.normal_rate = nor_amount;
        Ok(())
    }

    pub fn initialize_fixed_pool(ctx: Context<InitializeFixedPool>) -> ProgramResult {
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_init()?;
        fixed_pool.owner = ctx.accounts.owner.key();
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn stake_nft_to_fixed(
        ctx: Context<StakeNftToFixed>,
        global_bump: u8,
        lock_period: i64,
        role: String,
        model: u64,
    ) -> ProgramResult {
        let mint_metadata = &mut &ctx.accounts.mint_metadata;

        msg!("Metadata Account: {:?}", ctx.accounts.mint_metadata.key());
        let (metadata, _) = Pubkey::find_program_address(
            &[
                metaplex_token_metadata::state::PREFIX.as_bytes(),
                metaplex_token_metadata::id().as_ref(),
                ctx.accounts.nft_mint.key().as_ref(),
            ],
            &metaplex_token_metadata::id(),
        );
        require!(
            metadata == mint_metadata.key(),
            StakingError::InvaliedMetadata
        );

        // verify metadata is legit
        let nft_metadata = Metadata::from_account_info(mint_metadata)?;

        if let Some(creators) = nft_metadata.data.creators {
            let mut valid: u8 = 0;
            for creator in creators {
                if creator.address == CREATOR.parse::<Pubkey>().unwrap() && creator.verified == true
                {
                    valid = 1;
                    break;
                }
            }
            if valid != 1 {
                return Err(ProgramError::from(StakingError::InvalidCollection));
            }
        } else {
            return Err(ProgramError::from(StakingError::MetadataCreatorParseError));
        };

        let global_authority = &mut ctx.accounts.global_authority;

        let timestamp = Clock::get()?.unix_timestamp;
        let lock_time = timestamp + DAY * lock_period;

        let mut rate: i64 = 0;
        if model == 1 {
            match role.as_str() {
                ADV => rate = global_authority.adventure_rate as i64,
                SCI => rate = global_authority.scientist_rate as i64,
                DOC => rate = global_authority.doctor_rate as i64,
                SPE => rate = global_authority.specialist_rate as i64,
                COM => rate = global_authority.commander_rate as i64,
                _ => rate = 0,
            }
        }
        if model == 2 {
            rate = global_authority.normal_rate as i64;
        }
        if model == 3 {
            match lock_period {
                ONE => rate = global_authority.normal_rate as i64,
                TWO => rate = (global_authority.normal_rate * 15 / 10) as i64,
                FOUR => rate = (global_authority.normal_rate * 2) as i64,
                _ => rate = 0,
            }
        }
        let staked_item = StakedNFT {
            nft_addr: ctx.accounts.nft_mint.key(),
            stake_time: timestamp,
            reward_time: timestamp,
            lock_time: lock_time,
            rate: rate,
            model: model,
        };
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        fixed_pool.add_nft(staked_item);

        ctx.accounts.global_authority.total_amount += 1;
        let token_account_info = &mut &ctx.accounts.user_token_account;
        let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;

        let cpi_accounts = Transfer {
            from: token_account_info.to_account_info().clone(),
            to: dest_token_account_info.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        token::transfer(
            CpiContext::new(token_program.clone().to_account_info(), cpi_accounts),
            1,
        )?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn withdraw_nft_from_fixed(
        ctx: Context<WithdrawNftFromFixed>,
        global_bump: u8,
    ) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        let reward: u64 = fixed_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(),
            timestamp,
        )?;

        fixed_pool.pending_reward += reward;

        ctx.accounts.global_authority.total_amount -= 1;

        let token_account_info = &mut &ctx.accounts.user_token_account;
        let dest_token_account_info = &mut &ctx.accounts.dest_nft_token_account;
        let token_program = &mut &ctx.accounts.token_program;
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: dest_token_account_info.to_account_info().clone(),
            to: token_account_info.to_account_info().clone(),
            authority: ctx.accounts.global_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone().to_account_info(),
                cpi_accounts,
                signer,
            ),
            1,
        )?;
        /*
                sol_transfer_with_signer(
                    ctx.accounts.pool_wallet.to_account_info(),
                    ctx.accounts.owner.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    &[&[POOL_WALLET_SEED.as_ref(), &[pool_wallet_bump]]],
                    reward
                )?;
        */
        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn claim_reward_all(ctx: Context<ClaimRewardAll>, global_bump: u8) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        let reward: u64 = fixed_pool.claim_reward_all(timestamp)?;
        msg!("Reward: {}", reward);
        if ctx.accounts.reward_vault.amount < 1000_000_000 + reward {
            return Err(StakingError::LackLamports.into());
        }
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
            reward,
        )?;

        Ok(())
    }

    #[access_control(user(&ctx.accounts.user_fixed_pool, &ctx.accounts.owner))]
    pub fn claim_reward(ctx: Context<ClaimReward>, global_bump: u8) -> ProgramResult {
        let timestamp = Clock::get()?.unix_timestamp;
        let mut fixed_pool = ctx.accounts.user_fixed_pool.load_mut()?;
        let reward: u64 = fixed_pool.claim_reward(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(),
            timestamp,
        )?;
        msg!("Reward: {}", reward);
        if ctx.accounts.reward_vault.amount < 1000_000_000 + reward {
            return Err(StakingError::LackLamports.into());
        }
        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let token_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer),
            reward,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct SetAmount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,
}

#[derive(Accounts)]
pub struct InitializeFixedPool<'info> {
    #[account(zero)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct StakeNftToFixed<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,
    #[account(
        mut,
        constraint = user_token_account.mint == *nft_mint.to_account_info().key,
        constraint = user_token_account.owner == *owner.key,
        constraint = user_token_account.amount == 1,
    )]
    pub user_token_account: CpiAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = dest_nft_token_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_nft_token_account.owner == *global_authority.to_account_info().key,
    )]
    pub dest_nft_token_account: CpiAccount<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,
    #[account(
        mut,
        constraint = mint_metadata.owner == &metaplex_token_metadata::ID
    )]
    pub mint_metadata: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,

    // the token metadata program
    #[account(constraint = token_metadata_program.key == &metaplex_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct WithdrawNftFromFixed<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = user_token_account.mint == *nft_mint.to_account_info().key,
        constraint = user_token_account.owner == *owner.key,
    )]
    pub user_token_account: CpiAccount<'info, TokenAccount>,
    #[account(
        mut,
        constraint = dest_nft_token_account.mint == *nft_mint.to_account_info().key,
        constraint = dest_nft_token_account.owner == *global_authority.to_account_info().key,
        constraint = dest_nft_token_account.amount == 1,
    )]
    pub dest_nft_token_account: CpiAccount<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimRewardAll<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub user_fixed_pool: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        constraint = reward_vault.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = reward_vault.owner == global_authority.key(),
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_reward_account.mint == REWARD_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
        constraint = user_reward_account.owner == owner.key(),
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

// Access control modifiers
fn user(pool_loader: &AccountLoader<UserPool>, user: &AccountInfo) -> Result<()> {
    let user_pool = pool_loader.load()?;
    require!(user_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}
