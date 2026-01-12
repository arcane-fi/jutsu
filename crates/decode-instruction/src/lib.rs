// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use hayabusa_errors::Result;

pub trait DecodeIx<'ix>: Sized {
    fn decode(bytes: &'ix [u8]) -> Result<Self>;
}
