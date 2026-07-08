// CiosRowOps implementation for Uint<LIMBS>.
//
// `Uint<LIMBS>` stores `LIMBS` limbs of type `Limb(Word)` in little-endian
// order (limbs[0] = least significant). `Word` is `u64` on 64-bit platforms
// and `u32` on 32-bit platforms.
//
// `Limb::mac(acc, b, c, carry)` computes `acc + b*c + carry` and returns
// `(lo, new_carry)` — the exact CIOS multiply-accumulate kernel.

use crate::{Limb, Uint};
use modmath_cios::CiosRowOps;

impl<const LIMBS: usize> CiosRowOps for Uint<LIMBS> {
    type Word = crate::Word;

    #[inline]
    fn word_count(&self) -> usize {
        LIMBS
    }

    /// Infallible. Caller guarantees `i < LIMBS`.
    #[inline]
    fn word(&self, i: usize) -> crate::Word {
        self.limbs.get(i).map(|l| l.0).unwrap_or(0)
    }

    /// CIOS phase 1: `acc += scalar * multiplicand + carry_in`. Returns carry-out.
    fn mul_acc_row(
        scalar: crate::Word,
        multiplicand: &Self,
        acc: &mut Self,
        carry_in: crate::Word,
    ) -> crate::Word {
        let scalar_limb = Limb(scalar);
        let mut carry = Limb(carry_in);
        let mut j = 0;
        while j < LIMBS {
            let (lo, new_carry) = acc.limbs[j].mac(scalar_limb, multiplicand.limbs[j], carry);
            acc.limbs[j] = lo;
            carry = new_carry;
            j += 1;
        }
        carry.0
    }

    /// CIOS phase 2: `[acc, acc_hi] = ([acc, acc_hi] + scalar * multiplicand) >> word_bits`.
    /// Returns the carry word (0 or 1) from the fold.
    fn mul_acc_shift_row(
        scalar: crate::Word,
        multiplicand: &Self,
        acc: &mut Self,
        acc_hi: crate::Word,
    ) -> crate::Word {
        let scalar_limb = Limb(scalar);

        // Word 0: compute and discard; keep carry.
        let (_, mut carry) = Limb::ZERO.mac(scalar_limb, multiplicand.limbs[0], Limb(0));
        // Subtract the product of acc[0] separately — no, actually we include acc[0].
        // mac(acc[0], scalar, mult[0], 0) — discard the low word, keep carry.
        let (_, mut carry) = acc.limbs[0].mac(scalar_limb, multiplicand.limbs[0], Limb::ZERO);

        // Words 1..LIMBS: compute and shift down by one position.
        let mut j = 1;
        while j < LIMBS {
            let (lo, new_carry) = acc.limbs[j].mac(scalar_limb, multiplicand.limbs[j], carry);
            acc.limbs[j - 1] = lo;
            carry = new_carry;
            j += 1;
        }

        // Fold acc_hi + carry into acc[LIMBS-1]; return overflow bit (0 or 1).
        let (sum, overflow) = carry.adc(Limb(acc_hi), Limb::ZERO);
        acc.limbs[LIMBS - 1] = sum;
        overflow.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{U128, U256};
    use modmath_cios::CiosRowOps;

    #[test]
    fn word_count() {
        assert_eq!(CiosRowOps::word_count(&U128::ZERO), 2);
        assert_eq!(CiosRowOps::word_count(&U256::ZERO), 4);
    }

    #[test]
    fn word_roundtrip() {
        let v = U128::from_u64(0x1234567890ABCDEFu64);
        assert_eq!(CiosRowOps::word(&v, 0), 0x1234567890ABCDEFu64);
        assert_eq!(CiosRowOps::word(&v, 1), 0);
    }

    #[test]
    fn mul_acc_row_zero_scalar() {
        let mult = U128::ONE;
        let mut acc = U128::ONE;
        let carry = <U128 as CiosRowOps>::mul_acc_row(0, &mult, &mut acc, 0);
        assert_eq!(acc, U128::ONE);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mul_acc_row_no_carry() {
        // 3 * 4 = 12, fits in one limb.
        let mult = U128::from_u64(4);
        let mut acc = U128::ZERO;
        let carry = <U128 as CiosRowOps>::mul_acc_row(3, &mult, &mut acc, 0);
        assert_eq!(CiosRowOps::word(&acc, 0), 12);
        assert_eq!(CiosRowOps::word(&acc, 1), 0);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mul_acc_row_matches_u64_primitive() {
        // Compare Uint<2> (128-bit) single-limb op against the primitive u64 impl.
        let scalar: u64 = 0x123456789ABCDEF0;
        let mult_prim: u64 = 0x0FEDCBA987654321;
        let mut acc_prim: u64 = 0x1111111111111111;
        let carry_prim = <u64 as CiosRowOps>::mul_acc_row(scalar, &mult_prim, &mut acc_prim, 0);

        // Uint<1> is a 64-bit bigint (1 limb).
        type U64Big = Uint<1>;
        let mult_big = U64Big::from_u64(mult_prim);
        let mut acc_big = U64Big::from_u64(0x1111111111111111u64);
        let carry_big = <U64Big as CiosRowOps>::mul_acc_row(scalar, &mult_big, &mut acc_big, 0);

        assert_eq!(CiosRowOps::word(&acc_big, 0), acc_prim);
        assert_eq!(carry_big, carry_prim);
    }

    #[test]
    fn mul_acc_shift_row_zero() {
        let mult = U128::ZERO;
        let mut acc = U128::ZERO;
        let overflow = <U128 as CiosRowOps>::mul_acc_shift_row(0, &mult, &mut acc, 0);
        assert_eq!(acc, U128::ZERO);
        assert_eq!(overflow, 0);
    }

    #[test]
    fn mul_acc_shift_row_no_overflow() {
        // scalar=1, mult[0]=u64::MAX, acc[0]=0, acc_hi=0
        // mac(0, 1, MAX, 0) → lo = MAX, carry = 0
        // Discard lo (=MAX), carry = 0.
        // No more words (for Uint<1>). acc_hi=0 + carry=0 → acc[0]=0, overflow=0.
        type U64Big = Uint<1>;
        let mult = U64Big::from_u64(u64::MAX);
        let mut acc = U64Big::ZERO;
        let overflow = <U64Big as CiosRowOps>::mul_acc_shift_row(1, &mult, &mut acc, 0);
        assert_eq!(CiosRowOps::word(&acc, 0), 0);
        assert_eq!(overflow, 0);
    }
}
