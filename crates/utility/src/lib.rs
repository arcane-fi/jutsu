// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

#[macro_use]
pub mod macros;

use jutsu_errors::Result;
use pinocchio::{pubkey::Pubkey, program_error::ProgramError};

pub trait Len
where 
    Self: Sized,
{
    const DISCRIMINATED_LEN: usize = 8 + core::mem::size_of::<Self>();
}

#[inline(always)]
pub fn take_bytes<'a>(data: &'a [u8], n: usize) -> Result<(&'a [u8], &'a [u8])> {
    if data.len() < n {
        fail_with_ctx!(
            "JUTSU_TAKE_BYTES_INSUFFICIENT_DATA",
            ProgramError::InvalidInstructionData,
        );
    }
    Ok(data.split_at(n))
}


pub trait OwnerProgram {
    const OWNER: Pubkey;

    fn owner() -> Pubkey {
        Self::OWNER
    }
}