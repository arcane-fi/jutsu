// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use hayabusa_errors::{ErrorCode, Result, ProgramError};
use hayabusa_utility::{error_msg, hint::unlikely};
use hayabusa_common::AccountView;

pub trait FromAccountViews<'ix>
where
    Self: Sized,
{
    fn try_from_account_views(account_views: &mut AccountIter<'ix>) -> Result<Self>;
}

/// ## Context
///
/// A context consists of a set of typed/named accounts `T`
/// with constraints applied and a remaining accounts slice
pub struct Ctx<'ix, T>
where
    T: FromAccountViews<'ix>,
{
    pub accounts: T,
    pub remaining_accounts: &'ix [AccountView],
}

impl<'ix, T> Ctx<'ix, T>
where
    T: FromAccountViews<'ix>,
{
    #[inline(always)]
    pub fn construct(account_views: &'ix [AccountView]) -> Result<Self> {
        let mut iter = AccountIter::new(account_views);

        let accounts = T::try_from_account_views(&mut iter)?;

        Ok(Ctx {
            accounts,
            remaining_accounts: &account_views.get(iter.index..).unwrap_or(&[]),
        })
    }

    #[inline(always)]
    pub fn remaining_accounts(&self) -> AccountIter<'ix> {
        AccountIter::new(self.remaining_accounts)
    }
}

impl<'ix, T> core::ops::Deref for Ctx<'ix, T>
where
    T: FromAccountViews<'ix>,
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.accounts
    }
}

impl<'ix, T> core::ops::DerefMut for Ctx<'ix, T>
where
    T: FromAccountViews<'ix> + core::ops::Deref<Target = T>,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.accounts
    }
}

#[derive(Clone)]
pub struct AccountIter<'ix> {
    pub(crate) slice: &'ix [AccountView],
    pub(crate) index: usize,
}

impl<'ix> AccountIter<'ix> {
    #[inline(always)]
    pub fn new(slice: &'ix [AccountView]) -> Self {
        Self { slice, index: 0 }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> Result<&'ix AccountView> {
        if unlikely(self.index >= self.slice.len()) {
            error_msg!(
                "AccountIter::next: no accounts remaining.",
                ErrorCode::InvalidAccount,
            );
        }

        let account_view = &self.slice[self.index];
        self.index += 1;

        Ok(account_view)
    }

    #[inline(always)]
    pub fn into_subslice(&self) -> &[AccountView] {
        &self.slice[self.index..]
    }
}

pub struct AccountPeek<'ix> {
    slice: &'ix [AccountView],
    index: usize,
}

impl<'ix> From<&AccountIter<'ix>> for AccountPeek<'ix> {
    #[inline(always)]
    fn from(iter: &AccountIter<'ix>) -> AccountPeek<'ix> {
        AccountPeek {
            slice: iter.slice,
            index: iter.index,
        }
    }
}

impl<'ix> From<&mut AccountIter<'ix>> for AccountPeek<'ix> {
    #[inline(always)]
    fn from(iter: &mut AccountIter<'ix>) -> AccountPeek<'ix> {
        AccountPeek {
            slice: iter.slice,
            index: iter.index,
        }
    }
}

impl<'ix> AccountPeek<'ix> {
    #[inline(always)]
    pub fn set_index(&mut self, index: usize) -> Result<()> {
        if unlikely(index >= self.slice.len()) {
            error_msg!(
                "AccountPeek::set_index: index out of bounds.",
                ErrorCode::InvalidIndex,
            );
        }

        self.index = index;
         
        Ok(())
    }

    #[inline(always)]
    pub fn peek(&self, offset: usize) -> Result<&'ix AccountView> {
        if unlikely(self.index + offset >= self.slice.len()) {
            error_msg!(
                "AccountPeek::peek: index out of bounds.",
                ErrorCode::InvalidIndex,
            );
        }

        Ok(&self.slice[self.index + offset])
    }
}