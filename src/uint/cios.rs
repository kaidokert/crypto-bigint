// CiosRowOps for Uint<LIMBS>.
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

    #[inline]
    fn word(&self, i: usize) -> crate::Word {
        self.limbs.get(i).map_or(0, |l| l.0)
    }

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
            let (lo, new_carry) =
                scalar_limb.carrying_mul_add(multiplicand.limbs[j], acc.limbs[j], carry);
            acc.limbs[j] = lo;
            carry = new_carry;
            j += 1;
        }
        carry.0
    }

    fn mul_acc_shift_row(
        scalar: crate::Word,
        multiplicand: &Self,
        acc: &mut Self,
        acc_hi: crate::Word,
    ) -> crate::Word {
        let scalar_limb = Limb(scalar);

        // Word 0: compute, discard low word, keep carry.
        let (_, mut carry) =
            scalar_limb.carrying_mul_add(multiplicand.limbs[0], acc.limbs[0], Limb::ZERO);

        // Words 1..LIMBS: compute and shift down by one position.
        let mut j = 1;
        while j < LIMBS {
            let (lo, new_carry) =
                scalar_limb.carrying_mul_add(multiplicand.limbs[j], acc.limbs[j], carry);
            acc.limbs[j - 1] = lo;
            carry = new_carry;
            j += 1;
        }

        // Fold acc_hi + carry into acc[LIMBS-1]; return overflow bit (0 or 1).
        let (sum, overflow) = carry.carrying_add(Limb(acc_hi), Limb::ZERO);
        acc.limbs[LIMBS - 1] = sum;
        overflow.0
    }
}
