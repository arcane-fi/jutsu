// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use hayabusa_errors::Result;
use pinocchio::program_error::ProgramError;
use bytemuck::Pod;

pub trait DecodeIx<'a> {
    type Target;

    fn decode(bytes: &'a [u8]) -> Result<Self::Target>;
}

impl<'a, T> DecodeIx<'a> for T
where 
    T: Pod,
{
    type Target = &'a T;

    fn decode(bytes: &'a [u8]) -> Result<&'a T> {
        bytemuck::try_from_bytes::<T>(bytes)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

impl<'a> DecodeIx<'a> for [u8] {
    type Target = &'a [u8];

    fn decode(bytes: &'a [u8]) -> Result<&'a [u8]> {
        Ok(bytes)
    }
}