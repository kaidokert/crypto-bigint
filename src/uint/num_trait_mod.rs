use super::UInt;
use crate::Wrapping;

use num_traits::Bounded;
use num_traits::PrimInt;
use num_traits::Saturating;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use num_traits::{Num, NumCast, ToPrimitive};
use num_traits::{One, Zero};

impl<const LIMBS: usize> Bounded for Wrapping<UInt<LIMBS>> {
    fn min_value() -> Self {
        todo!()
    }
    fn max_value() -> Self {
        todo!()
    }
}

impl<const LIMBS: usize> CheckedAdd for Wrapping<UInt<LIMBS>> {
    fn checked_add(&self, _: &Self) -> Option<Self> {
        todo!()
    }
}
impl<const LIMBS: usize> CheckedSub for Wrapping<UInt<LIMBS>> {
    fn checked_sub(&self, _: &Self) -> Option<Self> {
        todo!()
    }
}
impl<const LIMBS: usize> CheckedMul for Wrapping<UInt<LIMBS>> {
    fn checked_mul(&self, _: &Self) -> Option<Self> {
        todo!()
    }
}
impl<const LIMBS: usize> CheckedDiv for Wrapping<UInt<LIMBS>> {
    fn checked_div(&self, _: &Self) -> Option<Self> {
        todo!()
    }
}

impl<const LIMBS: usize> One for Wrapping<UInt<LIMBS>> {
    fn one() -> Self {
        todo!()
    }
}
impl<const LIMBS: usize> Zero for Wrapping<UInt<LIMBS>> {
    fn zero() -> Self {
        todo!()
    }
    fn is_zero(&self) -> bool {
        todo!()
    }
}

impl<const LIMBS: usize> Saturating for Wrapping<UInt<LIMBS>> {
    fn saturating_add(self, _: Self) -> Self {
        todo!()
    }
    fn saturating_sub(self, _: Self) -> Self {
        todo!()
    }
}

impl<const LIMBS: usize> Num for Wrapping<UInt<LIMBS>> {
    type FromStrRadixErr = core::num::ParseIntError;
    fn from_str_radix(
        _: &str,
        _: u32,
    ) -> core::result::Result<Self, <Self as num_traits::Num>::FromStrRadixErr> {
        todo!()
    }
}

impl<const LIMBS: usize> ToPrimitive for Wrapping<UInt<LIMBS>> {
    fn to_i64(&self) -> Option<i64> {
        todo!()
    }
    fn to_u64(&self) -> Option<u64> {
        todo!()
    }
}

impl<const LIMBS: usize> NumCast for Wrapping<UInt<LIMBS>> {
    fn from<T>(_: T) -> Option<Self>
    where
        T: ToPrimitive,
    {
        todo!()
    }
}

impl<const LIMBS: usize> PrimInt for Wrapping<UInt<LIMBS>> {
    fn trailing_zeros(self) -> u32 {
        todo!()
    }
    fn count_ones(self) -> u32 {
        todo!()
    }
    fn count_zeros(self) -> u32 {
        todo!()
    }
    fn leading_zeros(self) -> u32 {
        todo!()
    }
    fn rotate_left(self, _: u32) -> Self {
        todo!()
    }
    fn rotate_right(self, _: u32) -> Self {
        todo!()
    }
    fn signed_shl(self, _: u32) -> Self {
        todo!()
    }
    fn signed_shr(self, _: u32) -> Self {
        todo!()
    }
    fn unsigned_shl(self, _: u32) -> Self {
        todo!()
    }
    fn unsigned_shr(self, _: u32) -> Self {
        todo!()
    }
    fn swap_bytes(self) -> Self {
        todo!()
    }
    fn from_be(_: Self) -> Self {
        todo!()
    }
    fn from_le(_: Self) -> Self {
        todo!()
    }
    fn to_be(self) -> Self {
        todo!()
    }
    fn to_le(self) -> Self {
        todo!()
    }
    fn pow(self, _: u32) -> Self {
        todo!()
    }
}
