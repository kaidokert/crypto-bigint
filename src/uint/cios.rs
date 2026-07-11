// CiosRowOps for Uint<LIMBS>.
//
// Uses Limb::carrying_mul_add (self * rhs + addend + carry → (lo, hi)) and
// Limb::carrying_add (self + rhs + carry → (lo, hi)) from crypto-bigint's
// own Limb type — no deprecated mac/adc calls.
//
// Limbs are little-endian: limbs[0] is the least significant word.

use crate::{Limb, Uint, Word};
use modmath_cios::CiosRowOps;

impl<const LIMBS: usize> CiosRowOps for Uint<LIMBS> {
    type Word = Word;

    #[inline]
    fn word_count(&self) -> usize {
        LIMBS
    }

    #[inline]
    fn word(&self, i: usize) -> Word {
        self.limbs[i].0
    }

    fn mul_acc_row(scalar: Word, multiplicand: &Self, acc: &mut Self, carry_in: Word) -> Word {
        let scalar = Limb(scalar);
        let mut carry = Limb(carry_in);
        let mut i = 0;
        while i < LIMBS {
            let (lo, hi) = scalar.carrying_mul_add(multiplicand.limbs[i], acc.limbs[i], carry);
            acc.limbs[i] = lo;
            carry = hi;
            i += 1;
        }
        carry.0
    }

    fn mul_acc_shift_row(scalar: Word, multiplicand: &Self, acc: &mut Self, acc_hi: Word) -> Word {
        let scalar = Limb(scalar);

        // Limb 0: compute scalar * multiplicand[0] + acc[0]; discard the low
        // word (it's the value being "shifted out"), keep the carry.
        let (_, carry0) = scalar.carrying_mul_add(multiplicand.limbs[0], acc.limbs[0], Limb::ZERO);
        let mut carry = carry0;

        // Limbs 1..LIMBS: compute and shift down by one position.
        let mut i = 1;
        while i < LIMBS {
            let (lo, hi) = scalar.carrying_mul_add(multiplicand.limbs[i], acc.limbs[i], carry);
            acc.limbs[i - 1] = lo;
            carry = hi;
            i += 1;
        }

        // Fold acc_hi + carry into acc[LIMBS-1]; return the overflow bit (0 or 1).
        let (sum, overflow) = Limb(acc_hi).carrying_add(carry, Limb::ZERO);
        acc.limbs[LIMBS - 1] = sum;
        overflow.0
    }
}
