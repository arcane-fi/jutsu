// Copyright (c) 2026, Arcane Labs <dev@arcane.fi>
// SPDX-License-Identifier: Apache-2.0

#![no_std]

use hayabusa_common::Address;

pub trait EventField {
    const SIZE: usize;

    fn write(&self, buf: &mut [u8]);
}

pub trait EventBuilder {
    fn emit(&self);
}

/// Emit a hex-encoded event log
#[macro_export]
macro_rules! emit {
    ($event:expr) => {
        $event.emit();
    };
}

#[macro_export]
macro_rules! impl_event_field_int {
    ($t:ty) => {
        impl EventField for $t {
            const SIZE: usize = core::mem::size_of::<$t>();

            #[inline(always)]
            fn write(&self, buf: &mut [u8]) {
                buf.copy_from_slice(&self.to_le_bytes());
            }
        }
    };
}

impl_event_field_int!(u8);
impl_event_field_int!(u16);
impl_event_field_int!(u32);
impl_event_field_int!(u64);
impl_event_field_int!(u128);

impl EventField for Address {
    const SIZE: usize = 32;

    #[inline(always)]
    fn write(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.as_ref());
    }
}

impl<const N: usize> EventField for [u8; N] {
    const SIZE: usize = N;

    #[inline(always)]
    fn write(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self);
    }
}