// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use hayabusa_accounts::ProgramIds;
use pinocchio::pubkey::Pubkey;

pub struct TokenInterface;

impl ProgramIds for TokenInterface {
    const IDS: &'static [Pubkey] = &[hayabusa_token::ID, hayabusa_token2022::ID];
}
