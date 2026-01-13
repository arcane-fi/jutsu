// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

pub mod zc;

use core::ops::Deref;
use hayabusa_common::{AccountView, Ref, RefMut};
use hayabusa_errors::Result;
pub use zc::*;

// marker traits

pub trait Zc {}
pub trait Deserialize {}
pub trait DeserializeMut {}

pub trait ConstructRef<'ix, R>
where
    R: Deref<Target = Self> + 'ix,
    Self: Sized,
{
    fn construct_ref(account_view: &'ix AccountView) -> Result<R>;
}

impl<'ix, T> ConstructRef<'ix, Ref<'ix, T>> for T
where
    T: ZcDeserialize,
{
    #[inline(always)]
    fn construct_ref(account_view: &'ix AccountView) -> Result<Ref<'ix, T>> {
        T::try_deserialize(account_view)
    }
}

impl<'ix, T> ConstructRef<'ix, RefMut<'ix, T>> for T
where
    T: ZcDeserializeMut,
{
    #[inline(always)]
    fn construct_ref(account_view: &'ix AccountView) -> Result<RefMut<'ix, T>> {
        T::try_deserialize_mut(account_view)
    }
}

impl<'ix, T> ConstructRef<'ix, &'ix T> for T
where
    T: RawZcDeserializeUnchecked,
{
    /// # Safety
    /// This method is unsafe to call if T does not uphold the contracts requried by
    /// RawZcDeserializeUnchecked, and if the callsite doesn't uphold the correct contracts
    #[inline(always)]
    fn construct_ref(account_view: &'ix AccountView) -> Result<&'ix T> {
        unsafe { T::try_deserialize_raw_unchecked(account_view) }
    }
}

impl<'ix, T> ConstructRef<'ix, &'ix mut T> for T
where
    T: RawZcDeserializeUncheckedMut,
{
    #[inline(always)]
    fn construct_ref(account_view: &'ix AccountView) -> Result<&'ix mut T> {
        unsafe { T::try_deserialize_raw_unchecked_mut(account_view) }
    }
}
