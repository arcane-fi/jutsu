// Copyright (c) 2025, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

use crate::{FromAccountInfo, Key, ProgramIds, ToAccountInfo};
use core::ops::Deref;
use hayabusa_errors::{ErrorCode, Result};
use hayabusa_utility::error_msg;
use pinocchio::{account_info::AccountInfo, hint::unlikely, pubkey::Pubkey};

pub struct Interface<'ix, T>
where
    T: ProgramIds,
{
    pub account_info: &'ix AccountInfo,
    _phantom: core::marker::PhantomData<T>,
}

impl<'ix, T> FromAccountInfo<'ix> for Interface<'ix, T>
where
    T: ProgramIds,
{
    #[inline(always)]
    fn try_from_account_info(account_info: &'ix AccountInfo) -> Result<Self> {
        if unlikely(!T::IDS.contains(account_info.key())) {
            error_msg!(
                "Interface::try_from_account_info: program ID mismatch",
                ErrorCode::InvalidProgram,
            );
        }

        Ok(Interface {
            account_info,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<T> ToAccountInfo for Interface<'_, T>
where
    T: ProgramIds,
{
    #[inline(always)]
    fn to_account_info(&self) -> &AccountInfo {
        self.account_info
    }
}

impl<T> Key for Interface<'_, T>
where
    T: ProgramIds,
{
    #[inline(always)]
    fn key(&self) -> &Pubkey {
        self.account_info.key()
    }
}

impl<T> Deref for Interface<'_, T>
where
    T: ProgramIds,
{
    type Target = AccountInfo;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.account_info
    }
}
