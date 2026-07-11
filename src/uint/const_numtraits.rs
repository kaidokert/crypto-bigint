// const-num-traits 0.2 impls for Uint<LIMBS>.
//
// Compiled unconditionally — sits alongside the existing num-traits impls in
// uint.rs without touching them. Every inherent method call uses UFCS to
// avoid method-resolution ambiguity when both trait and inherent are in scope.

use crate::{Limb, Uint};

// ── Identity values ──────────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::Zero for Uint<LIMBS> {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        bool::from(Uint::is_zero(self))
    }
    fn set_zero(&mut self) {
        *self = Self::ZERO;
    }
}

impl<const LIMBS: usize> const_num_traits::One for Uint<LIMBS> {
    fn one() -> Self {
        Self::ONE
    }
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
    fn set_one(&mut self) {
        *self = Self::ONE;
    }
}

// ── Personality ───────────────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::HasPersonality for Uint<LIMBS> {
    type P = const_num_traits::Nct;
}

// ── Wrapping arithmetic ───────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::ops::wrapping::WrappingAdd for Uint<LIMBS> {
    type Output = Self;
    fn wrapping_add(self, v: Self) -> Self {
        Uint::wrapping_add(&self, &v)
    }
}

impl<const LIMBS: usize> const_num_traits::ops::wrapping::WrappingSub for Uint<LIMBS> {
    type Output = Self;
    fn wrapping_sub(self, v: Self) -> Self {
        Uint::wrapping_sub(&self, &v)
    }
}

impl<const LIMBS: usize> const_num_traits::ops::wrapping::WrappingMul for Uint<LIMBS> {
    type Output = Self;
    fn wrapping_mul(self, v: Self) -> Self {
        Uint::wrapping_mul(&self, &v)
    }
}

// ── Overflowing arithmetic ────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::ops::overflowing::OverflowingAdd for Uint<LIMBS> {
    type Output = Self;
    fn overflowing_add(self, v: Self) -> (Self, bool) {
        let (result, carry) = Uint::carrying_add(&self, &v, Limb::ZERO);
        (result, carry != Limb::ZERO)
    }
}

// ── Carrying / borrowing arithmetic ──────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::BorrowingSub for Uint<LIMBS> {
    type Output = Self;
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (result, b) = Uint::borrowing_sub(&self, &rhs, Limb::from(borrow as u8));
        (result, b != Limb::ZERO)
    }
}

impl<const LIMBS: usize> const_num_traits::CarryingMul for Uint<LIMBS> {
    type Unsigned = Self;
    type Output = Self;

    fn carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        let (lo, hi) = Uint::widening_mul(&self, &rhs);
        let (lo, c) = Uint::carrying_add(&lo, &carry, Limb::ZERO);
        let hi = Uint::wrapping_add(&hi, &Uint::from_word(c.0));
        (lo, hi)
    }

    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self) {
        let (lo, hi) = Uint::widening_mul(&self, &rhs);
        let (lo, c1) = Uint::carrying_add(&lo, &carry, Limb::ZERO);
        let (lo, c2) = Uint::carrying_add(&lo, &add, Limb::ZERO);
        let hi = Uint::wrapping_add(&hi, &Uint::from_word(c1.0.wrapping_add(c2.0)));
        (lo, hi)
    }
}

// ── Parity ────────────────────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::Parity for Uint<LIMBS> {
    fn is_odd(self) -> bool {
        self.limbs[0].0 & 1 == 1
    }
    fn is_even(self) -> bool {
        self.limbs[0].0 & 1 == 0
    }
}

// ── CT traits ─────────────────────────────────────────────────────────────────

#[cfg(feature = "subtle")]
impl<const LIMBS: usize> const_num_traits::ops::ct::CtIsZero for Uint<LIMBS> {
    fn ct_is_zero(&self) -> subtle::Choice {
        use subtle::ConstantTimeEq;
        self.ct_eq(&Self::ZERO)
    }
}
