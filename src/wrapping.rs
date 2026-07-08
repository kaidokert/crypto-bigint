//! Wrapping arithmetic.

use crate::{WrappingAdd, WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub, Zero};
use core::{
    fmt,
    ops::{Add, Mul, Neg, Shl, Shr, Sub},
};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

#[cfg(feature = "rand_core")]
use {crate::Random, rand_core::RngCore};

#[cfg(feature = "serde")]
use serdect::serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Provides intentionally-wrapped arithmetic on `T`.
///
/// This is analogous to [`core::num::Wrapping`] but allows this crate to
/// define trait impls for this type.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Wrapping<T>(pub T);

impl<T: WrappingAdd + Add<Output = T> + Clone> Add<Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_add(rhs.0.clone()))
    }
}

impl<T: WrappingAdd + Add<Output = T> + Clone> Add<&Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn add(self, rhs: &Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_add(rhs.0.clone()))
    }
}

impl<T: WrappingAdd + Add<Output = T> + Clone> Add<Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn add(self, rhs: Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_add(rhs.0.clone()))
    }
}

impl<T: WrappingAdd + Add<Output = T> + Clone> Add<&Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn add(self, rhs: &Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_add(rhs.0.clone()))
    }
}

impl<T: WrappingSub + Sub<Output = T> + Clone> Sub<Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_sub(rhs.0.clone()))
    }
}

impl<T: WrappingSub + Sub<Output = T> + Clone> Sub<&Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn sub(self, rhs: &Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_sub(rhs.0.clone()))
    }
}

impl<T: WrappingSub + Sub<Output = T> + Clone> Sub<Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn sub(self, rhs: Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_sub(rhs.0.clone()))
    }
}

impl<T: WrappingSub + Sub<Output = T> + Clone> Sub<&Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn sub(self, rhs: &Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_sub(rhs.0.clone()))
    }
}

impl<T: WrappingMul + Mul<Output = T> + Clone> Mul<Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_mul(rhs.0.clone()))
    }
}

impl<T: WrappingMul + Mul<Output = T> + Clone> Mul<&Self> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn mul(self, rhs: &Self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_mul(rhs.0.clone()))
    }
}

impl<T: WrappingMul + Mul<Output = T> + Clone> Mul<Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn mul(self, rhs: Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_mul(rhs.0.clone()))
    }
}

impl<T: WrappingMul + Mul<Output = T> + Clone> Mul<&Wrapping<T>> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn mul(self, rhs: &Wrapping<T>) -> Self::Output {
        Wrapping(self.0.clone().wrapping_mul(rhs.0.clone()))
    }
}

impl<T: WrappingNeg<Output = T> + Neg<Output = T> + Clone> Neg for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn neg(self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_neg())
    }
}

impl<T: WrappingNeg<Output = T> + Neg<Output = T> + Clone> Neg for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn neg(self) -> Self::Output {
        Wrapping(self.0.clone().wrapping_neg())
    }
}

impl<T: WrappingShl + Shl<usize, Output = T> + Clone> Shl<u32> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        Wrapping(self.0.clone().wrapping_shl(rhs))
    }
}

impl<T: WrappingShl + Shl<usize, Output = T> + Clone> Shl<u32> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        Wrapping(self.0.clone().wrapping_shl(rhs))
    }
}

impl<T: WrappingShr + Shr<usize, Output = T> + Clone> Shr<u32> for Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        Wrapping(self.0.clone().wrapping_shr(rhs))
    }
}

impl<T: WrappingShr + Shr<usize, Output = T> + Clone> Shr<u32> for &Wrapping<T> {
    type Output = Wrapping<T>;

    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        Wrapping(self.0.clone().wrapping_shr(rhs))
    }
}

impl<T: ConditionallySelectable> ConditionallySelectable for Wrapping<T> {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Wrapping(T::conditional_select(&a.0, &b.0, choice))
    }
}

impl<T: ConstantTimeEq> ConstantTimeEq for Wrapping<T> {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl<T: Zero> Zero for Wrapping<T> {
    #[inline]
    fn zero() -> Self {
        Wrapping(T::zero())
    }
}

impl<T: num_traits::Zero + WrappingAdd + Add<Output = T> + Clone> num_traits::Zero for Wrapping<T> {
    #[inline]
    fn zero() -> Self {
        Wrapping(T::zero())
    }

    #[inline]
    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: num_traits::One + WrappingMul + Mul<Output = T> + PartialEq + Clone> num_traits::One for Wrapping<T> {
    #[inline]
    fn one() -> Self {
        Wrapping(T::one())
    }

    #[inline]
    fn set_one(&mut self) {
        self.0.set_one();
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl<T: fmt::Display> fmt::Display for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Binary> fmt::Binary for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Octal> fmt::Octal for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::LowerHex> fmt::LowerHex for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::UpperHex> fmt::UpperHex for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "rand_core")]
impl<T: Random> Random for Wrapping<T> {
    fn random<R: RngCore + ?Sized>(rng: &mut R) -> Self {
        Wrapping(Random::random(rng))
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Wrapping<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for Wrapping<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(all(test, feature = "serde"))]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{Wrapping, U64};

    #[test]
    fn serde() {
        const TEST: Wrapping<U64> = Wrapping(U64::from_u64(0x0011223344556677));

        let serialized = bincode::serialize(&TEST).unwrap();
        let deserialized: Wrapping<U64> = bincode::deserialize(&serialized).unwrap();

        assert_eq!(TEST, deserialized);
    }

    #[test]
    fn serde_owned() {
        const TEST: Wrapping<U64> = Wrapping(U64::from_u64(0x0011223344556677));

        let serialized = bincode::serialize(&TEST).unwrap();
        let deserialized: Wrapping<U64> = bincode::deserialize_from(serialized.as_slice()).unwrap();

        assert_eq!(TEST, deserialized);
    }
}
