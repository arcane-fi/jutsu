// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use core::slice::from_raw_parts;
use hayabusa_cpi::{CheckProgramId, CpiCtx};
use hayabusa_errors::Result;
use hayabusa_utility::{write_uninit_bytes, UNINIT_BYTE};
use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke, invoke_signed},
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub struct BurnChecked<'ix> {
    /// The account being burned from
    pub burn_account: &'ix AccountInfo,
    /// The mint account
    pub mint: &'ix AccountInfo,
    /// The authority account
    pub authority: &'ix AccountInfo,
}

impl CheckProgramId for BurnChecked<'_> {
    const ID: Pubkey = crate::ID;
}

const DISCRIMINATOR: [u8; 1] = [15];

#[inline(always)]
pub fn burn_checked<'ix>(
    cpi_ctx: CpiCtx<'ix, '_, '_, '_, BurnChecked<'ix>>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let infos = [cpi_ctx.burn_account, cpi_ctx.mint, cpi_ctx.authority];

    let metas = [
        AccountMeta::writable(cpi_ctx.burn_account.key()),
        AccountMeta::writable(cpi_ctx.mint.key()),
        AccountMeta::readonly_signer(cpi_ctx.authority.key()),
    ];

    // ix data layout
    // - [0]: discriminator
    // - [1..9]: amount
    // - [9]: decimals
    let mut ix_data = [UNINIT_BYTE; 10];

    write_uninit_bytes(&mut ix_data, &DISCRIMINATOR);
    write_uninit_bytes(&mut ix_data[1..9], &amount.to_le_bytes());
    write_uninit_bytes(&mut ix_data[9..], &[decimals]);

    let ix = Instruction {
        program_id: &crate::ID,
        accounts: &metas,
        data: unsafe { from_raw_parts(ix_data.as_ptr() as _, 10) },
    };

    if let Some(signers) = cpi_ctx.signers {
        invoke_signed(&ix, &infos, signers)
    } else {
        invoke(&ix, &infos)
    }
}
