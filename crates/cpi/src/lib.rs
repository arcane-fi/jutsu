// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use pinocchio::{account_info::AccountInfo, instruction::Signer, pubkey::Pubkey, sysvars::{rent::Rent, Sysvar}};
use pinocchio_system::instructions::CreateAccount;
use jutsu_errors::Result;

pub fn create_account<'a>(
    program_id: &'a Pubkey,
    target_account: &'a AccountInfo,
    payer: &'a AccountInfo,
    signers: Option<&[Signer]>,
    space: u64,
) -> Result<()> {
    let rent = Rent::get()?;

    let min_lamports = rent.minimum_balance(space as usize);

    let create_account = CreateAccount {
        from: payer,
        to: target_account,
        lamports: min_lamports,
        space,
        owner: program_id,
    };

    if let Some(signers) = signers {
        create_account.invoke_signed(signers)?;
    } else {
        create_account.invoke()?;
    }

    Ok(())
}