#[allow(unexpected_cfgs)]

use anchor_lang::{prelude::*,system_program::{Transfer,transfer}};

declare_id!("FWdRhCmLTm7gBxF4zFEHmQBEyjQFxDznkfZaz3E6QtGu");

#[program]
pub mod anchor_vault_q1_2026 {
    use super::*;

		pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
			msg!("Rust bumps 1 : {}",ctx.bumps.vault_state);
			msg!("Rust bumps 2 : {}",ctx.bumps.vault);
			ctx.accounts.initialize(&ctx.bumps)
		}

		pub fn deposit(ctx: Context<Deposit>,amount: u64)-> Result<()> {
			ctx.accounts.deposit(amount)
		}

		pub fn withdraw(ctx: Context<Withdraw>,amount: u64)-> Result<()>{
			ctx.accounts.withdraw(amount)
		}

		pub fn close(ctx: Context<Close>)-> Result<()>{
			ctx.accounts.close()
		}
}

/// Instruction Handler Context types
///
// INITIALIZE
///
// Initialize structure
#[derive(Accounts)]
pub struct Initialize<'info> {
	pub system_program: Program<'info,System>,

	#[account(mut)]
	pub user: Signer<'info>,

	#[account(
		init,
		payer = user,
		seeds = [b"state", user.key().as_ref()],
		bump,
		space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
	)]
	pub vault_state: Account<'info,VaultState>,

	#[account(
		mut,
		seeds = [b"vault", vault_state.key().as_ref()],
		bump,
	)]
	pub vault: SystemAccount<'info>,

}

// Initialize structure implementations
impl<'info> Initialize<'info> {
	pub fn initialize(&mut self, bump: &InitializeBumps)-> Result<()> {

		let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());

		let cpi_program = self.system_program.to_account_info();

		let cpi_accounts = Transfer {
			from: self.user.to_account_info(),
			to: self.vault.to_account_info()
		};

		let cpi_ctx = CpiContext::new(cpi_program,cpi_accounts);

		transfer(cpi_ctx, rent_exempt)?;

		self.vault_state.state_bump = bump.vault_state;
		self.vault_state.vault_bump = bump.vault;

		Ok(())
	}
}

///
// DEPOSIT
///
// Deposit struct
#[derive(Accounts)]
pub struct Deposit<'info> {
	pub system_program: Program<'info,System>,

	#[account(mut)]
	pub user: Signer<'info>,

	#[account(
		seeds = [b"state", user.key().as_ref()],
		bump = vault_state.state_bump,
	)]
	pub vault_state: Account<'info,VaultState>,

	#[account(
		mut,
		seeds = [b"vault", vault_state.key().as_ref()],
		bump = vault_state.vault_bump,
	)]
	pub vault: SystemAccount<'info>,

}

// Deposit struct implementations
impl<'info> Deposit<'info> {
	pub fn deposit(&mut self, amount: u64)-> Result<()> {
		let cpi_program = self.system_program.to_account_info();
		let cpi_account = Transfer{
			from: self.user.to_account_info(),
			to: self.vault.to_account_info()
		};
		let cpi_ctx = CpiContext::new(cpi_program, cpi_account);

		transfer(cpi_ctx,amount)?;

		Ok(())
	}

}

///
// WITHDRAW
///
// Withdraw struct
#[derive(Accounts)]
pub struct Withdraw<'info> {
	pub system_program: Program<'info,System>,

	pub user: Signer<'info>,

	#[account(
		seeds = [b"state",user.key().as_ref()],
		bump = vault_state.state_bump
	)]
	pub vault_state: Account<'info,VaultState>,

	#[account(
		mut,
		seeds = [b"vault",vault_state.key().as_ref()],
		bump = vault_state.vault_bump
	)]
	pub vault: SystemAccount<'info>,
}

// Withdraw struct implementations
impl<'info> Withdraw<'info> {
	pub fn withdraw(&mut self,amount:u64)-> Result<()> {
		let cpi_program = self.system_program.to_account_info();

		let cpi_accounts = Transfer{
			from: self.vault.to_account_info(),
			to: self.user.to_account_info()
		};

		// let second_seed = self.vault_state.key();

		// Here , the vault_state address is given in the way it is,
		// i.e. self.vault_state.to_account_info().key.as_ref()
		// NOTE: 'key' is used as an attribute not a function here.
		// so that it lives till the end of the program or untile the lifetime of <'info>. 
		// using "self.vault_state.key()" gives a "temporary value dropped" error
		let cpi_seed_signer:&[&[&[u8]]] = &[&[b"vault",self.vault_state.to_account_info().key.as_ref(),&[self.vault_state.vault_bump]]];

		let cpi_ctx = CpiContext::new_with_signer(
			cpi_program,
			cpi_accounts,
			cpi_seed_signer
		);

		transfer(cpi_ctx, amount)?;

		Ok(())
	}
}

///
// CLOSE
///
// Close struct
#[derive(Accounts)]
pub struct Close<'info> {
	pub system_program: Program<'info,System>,

	pub user: Signer<'info>,

	#[account(
		mut,
		close = user,
		seeds = [b"state",user.key().as_ref()],
		bump = vault_state.state_bump
	)]
	pub vault_state: Account<'info,VaultState>,

	#[account(
		mut,
		seeds = [b"vault",vault_state.key().as_ref()],
		bump = vault_state.vault_bump
	)]
	pub vault: SystemAccount<'info>,
}

// Close struct implementations
impl<'info> Close<'info> {
	pub fn close(&mut self) -> Result<()>{
		let cpi_program = self.system_program.to_account_info();

		let cpi_accounts = Transfer{
			from: self.vault.to_account_info(),
			to: self.user.to_account_info()
		};

		// When I gave "self.vault_state.key().as_ref()" directly to  "cpi_seed_signer" 
		// the signer-seed object, I saw an error saying "temporary value dropped"
		// Not sure why that happned
		let second_seed = self.vault_state.key();

		let cpi_seed_signer:&[&[&[u8]]] = &[&[b"vault",second_seed.as_ref(),&[self.vault_state.vault_bump]]];

		let cpi_ctx = CpiContext::new_with_signer(
			cpi_program,
			cpi_accounts,
			cpi_seed_signer
		);

		let vault_leftover_balance = self.vault.to_account_info().lamports();

		transfer(cpi_ctx, vault_leftover_balance)?;

		Ok(())
	}
}

// Struct Types
#[derive(InitSpace)]
#[account]
pub struct VaultState {
	pub vault_bump: u8,
	pub state_bump: u8,
}
