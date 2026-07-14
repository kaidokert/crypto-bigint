// const-num-traits 0.2 impls for Uint<LIMBS>.
//
// Compiled unconditionally — sits alongside the existing num-traits impls in
// uint.rs without touching them. Every inherent method call uses UFCS to
// avoid method-resolution ambiguity when both trait and inherent are in scope.

use crate::{Limb, Uint, Word};
use core::mem::size_of_val;

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

impl<const LIMBS: usize> const_num_traits::ops::checked::CheckedAdd for Uint<LIMBS> {
    type Output = Self;
    fn checked_add(self, v: Self) -> Option<Self> {
        let (result, carry) = Uint::carrying_add(&self, &v, Limb::ZERO);
        if carry != Limb::ZERO { None } else { Some(result) }
    }
}

impl<const LIMBS: usize> const_num_traits::ops::checked::CheckedMul for Uint<LIMBS> {
    type Output = Self;
    fn checked_mul(self, v: Self) -> Option<Self> {
        let (lo, overflow) = self.overflowing_mul(&v);
        if bool::from(overflow) { None } else { Some(lo) }
    }
}

impl<const LIMBS: usize> const_num_traits::ops::bits::BitsPrecision for Uint<LIMBS> {
    fn bits_precision(self) -> u32 {
        Self::BITS
    }
}

impl<const LIMBS: usize> const_num_traits::ops::bits::WithPrecision for Uint<LIMBS> {
    fn widen_to_precision(self, _bits_precision: u32) -> Self {
        self
    }
}

impl<const LIMBS: usize> const_num_traits::ops::overflowing::OverflowingSub for Uint<LIMBS> {
    type Output = Self;
    fn overflowing_sub(self, v: Self) -> (Self, bool) {
        let (result, borrow) = Uint::borrowing_sub(&self, &v, Limb::ZERO);
        (result, borrow != Limb::ZERO)
    }
}

// ── Carrying / borrowing arithmetic ──────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::BorrowingSub for Uint<LIMBS> {
    type Output = Self;
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (result, b) = Uint::borrowing_sub(&self, &rhs, Limb::from(u8::from(borrow)));
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

// ── ConstZero / ConstOne (required by PrimBits) ───────────────────────────────

impl<const LIMBS: usize> const_num_traits::ConstZero for Uint<LIMBS> {
    const ZERO: Self = Self::ZERO;
}

impl<const LIMBS: usize> const_num_traits::ConstOne for Uint<LIMBS> {
    const ONE: Self = Self::ONE;
}

// ── PrimBits ─────────────────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::PrimBits for Uint<LIMBS> {
    fn count_ones(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS {
            n += self.limbs[i].0.count_ones();
            i += 1;
        }
        n
    }

    fn count_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS {
            n += self.limbs[i].0.count_zeros();
            i += 1;
        }
        n
    }

    fn leading_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = LIMBS;
        while i > 0 {
            i -= 1;
            let lz = self.limbs[i].0.leading_zeros();
            n += lz;
            if lz < Limb::BITS {
                break;
            }
        }
        n
    }

    fn trailing_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS {
            let tz = self.limbs[i].0.trailing_zeros();
            n += tz;
            if tz < Limb::BITS {
                break;
            }
            i += 1;
        }
        n
    }

    fn rotate_left(self, _n: u32) -> Self {
        todo!()
    }
    fn rotate_right(self, _n: u32) -> Self {
        todo!()
    }
    fn signed_shl(self, _n: u32) -> Self {
        todo!()
    }
    fn signed_shr(self, _n: u32) -> Self {
        todo!()
    }
    fn unsigned_shl(self, n: u32) -> Self {
        Uint::wrapping_shl(&self, n)
    }
    fn unsigned_shr(self, n: u32) -> Self {
        Uint::wrapping_shr(&self, n)
    }
    fn swap_bytes(self) -> Self {
        todo!()
    }
    fn from_be(x: Self) -> Self {
        todo!("{}", size_of_val(&x))
    }
    fn from_le(x: Self) -> Self {
        todo!("{}", size_of_val(&x))
    }
    fn to_be(self) -> Self {
        todo!()
    }
    fn to_le(self) -> Self {
        todo!()
    }
}

// ── Byte buffer ───────────────────────────────────────────────────────────────
//
// Fixed-size byte buffer: stores [Limb; LIMBS], exposes LIMBS*Limb::BYTES via
// unsafe ptr cast. Unifies ToBytes::Bytes == FromBytes::Bytes for
// FixedWidthUnsignedInt compat (rsa_heapless requires they be the same type).

#[derive(Clone, Copy)]
pub struct BytesHolder<const LIMBS: usize> {
    limbs: [Limb; LIMBS],
}

impl<const LIMBS: usize> Default for BytesHolder<LIMBS> {
    fn default() -> Self {
        Self {
            limbs: [Limb::ZERO; LIMBS],
        }
    }
}

impl<const LIMBS: usize> BytesHolder<LIMBS> {
    #[allow(unsafe_code)]
    #[inline]
    fn as_byte_slice(&self) -> &[u8] {
        // SAFETY: Limb is repr(transparent) over Word; array is valid for
        // reads of LIMBS * Limb::BYTES bytes from its base address.
        unsafe {
            core::slice::from_raw_parts(self.limbs.as_ptr().cast::<u8>(), LIMBS * Limb::BYTES)
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: unique &mut access; same size guarantee as as_byte_slice.
        unsafe {
            core::slice::from_raw_parts_mut(
                self.limbs.as_mut_ptr().cast::<u8>(),
                LIMBS * Limb::BYTES,
            )
        }
    }
}

impl<const LIMBS: usize> AsRef<[u8]> for BytesHolder<LIMBS> {
    fn as_ref(&self) -> &[u8] {
        self.as_byte_slice()
    }
}
impl<const LIMBS: usize> AsMut<[u8]> for BytesHolder<LIMBS> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_byte_slice_mut()
    }
}
impl<const LIMBS: usize> core::borrow::Borrow<[u8]> for BytesHolder<LIMBS> {
    fn borrow(&self) -> &[u8] {
        self.as_byte_slice()
    }
}
impl<const LIMBS: usize> core::borrow::BorrowMut<[u8]> for BytesHolder<LIMBS> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_byte_slice_mut()
    }
}
impl<const LIMBS: usize> PartialEq for BytesHolder<LIMBS> {
    fn eq(&self, other: &Self) -> bool {
        self.as_byte_slice() == other.as_byte_slice()
    }
}
impl<const LIMBS: usize> Eq for BytesHolder<LIMBS> {}
impl<const LIMBS: usize> PartialOrd for BytesHolder<LIMBS> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<const LIMBS: usize> Ord for BytesHolder<LIMBS> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_byte_slice().cmp(other.as_byte_slice())
    }
}
impl<const LIMBS: usize> core::hash::Hash for BytesHolder<LIMBS> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_byte_slice().hash(state);
    }
}
impl<const LIMBS: usize> core::fmt::Debug for BytesHolder<LIMBS> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BytesHolder(")?;
        for b in self.as_byte_slice() {
            write!(f, "{:02x}", b)?;
        }
        write!(f, ")")
    }
}
#[cfg(feature = "zeroize")]
impl<const LIMBS: usize> zeroize::DefaultIsZeroes for BytesHolder<LIMBS> {}

// ── ToBytes / FromBytes ───────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::ToBytes for Uint<LIMBS> {
    type Bytes = BytesHolder<LIMBS>;

    fn to_be_bytes(self) -> BytesHolder<LIMBS> {
        let mut holder = BytesHolder::default();
        let bytes = holder.as_byte_slice_mut();
        let limb_bytes = Limb::BYTES;
        let mut i = 0;
        while i < LIMBS {
            let word_be = self.limbs[LIMBS - 1 - i].0.to_be_bytes();
            let mut j = 0;
            while j < limb_bytes {
                bytes[i * limb_bytes + j] = word_be[j];
                j += 1;
            }
            i += 1;
        }
        holder
    }

    fn to_le_bytes(self) -> BytesHolder<LIMBS> {
        let mut holder = BytesHolder::default();
        let bytes = holder.as_byte_slice_mut();
        let limb_bytes = Limb::BYTES;
        let mut i = 0;
        while i < LIMBS {
            let word_le = self.limbs[i].0.to_le_bytes();
            let mut j = 0;
            while j < limb_bytes {
                bytes[i * limb_bytes + j] = word_le[j];
                j += 1;
            }
            i += 1;
        }
        holder
    }
}

impl<const LIMBS: usize> const_num_traits::ToBytes for &Uint<LIMBS> {
    type Bytes = BytesHolder<LIMBS>;

    fn to_be_bytes(self) -> BytesHolder<LIMBS> {
        const_num_traits::ToBytes::to_be_bytes(*self)
    }

    fn to_le_bytes(self) -> BytesHolder<LIMBS> {
        const_num_traits::ToBytes::to_le_bytes(*self)
    }
}

impl<const LIMBS: usize> const_num_traits::FromBytes for Uint<LIMBS> {
    type Bytes = BytesHolder<LIMBS>;

    fn from_be_bytes(bytes: &BytesHolder<LIMBS>) -> Self {
        let bytes = bytes.as_byte_slice();
        let mut limbs = [Limb::ZERO; LIMBS];
        let limb_bytes = Limb::BYTES;
        let mut i = 0;
        while i < LIMBS {
            let start = i * limb_bytes;
            let mut word: Word = 0;
            let mut j = 0;
            while j < limb_bytes {
                word = (word << 8) | Word::from(bytes[start + j]);
                j += 1;
            }
            limbs[LIMBS - 1 - i] = Limb(word);
            i += 1;
        }
        Uint::new(limbs)
    }

    fn from_le_bytes(bytes: &BytesHolder<LIMBS>) -> Self {
        let bytes = bytes.as_byte_slice();
        let mut limbs = [Limb::ZERO; LIMBS];
        let limb_bytes = Limb::BYTES;
        let mut i = 0;
        while i < LIMBS {
            let start = i * limb_bytes;
            let mut word: Word = 0;
            let mut j = 0;
            while j < limb_bytes {
                word |= Word::from(bytes[start + j]) << (j * 8);
                j += 1;
            }
            limbs[i] = Limb(word);
            i += 1;
        }
        Uint::new(limbs)
    }
}

// ── FromByteSlice ─────────────────────────────────────────────────────────────

impl<const LIMBS: usize> const_num_traits::FromByteSlice for Uint<LIMBS> {
    fn from_be_slice(
        bytes: &[u8],
    ) -> Result<Self, const_num_traits::ops::byte_slice::ByteSliceError> {
        use const_num_traits::ops::byte_slice::{ByteSliceError, ByteSliceErrorKind};
        if bytes.is_empty() {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Empty,
            });
        }
        let capacity = LIMBS * Limb::BYTES;
        if bytes.len() > capacity {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Overflow,
            });
        }
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut remaining = bytes;
        let mut limb_idx = 0usize;
        while limb_idx < LIMBS && !remaining.is_empty() {
            let chunk_len = if remaining.len() >= Limb::BYTES {
                Limb::BYTES
            } else {
                remaining.len()
            };
            let (head, tail) = remaining.split_at(remaining.len() - chunk_len);
            limbs[limb_idx] = Limb::from_be_slice(tail);
            remaining = head;
            limb_idx += 1;
        }
        Ok(Uint::new(limbs))
    }

    fn from_le_slice(
        bytes: &[u8],
    ) -> Result<Self, const_num_traits::ops::byte_slice::ByteSliceError> {
        use const_num_traits::ops::byte_slice::{ByteSliceError, ByteSliceErrorKind};
        if bytes.is_empty() {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Empty,
            });
        }
        let capacity = LIMBS * Limb::BYTES;
        if bytes.len() > capacity {
            return Err(ByteSliceError {
                kind: ByteSliceErrorKind::Overflow,
            });
        }
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut remaining = bytes;
        let mut limb_idx = 0usize;
        while limb_idx < LIMBS && !remaining.is_empty() {
            let chunk_len = if remaining.len() >= Limb::BYTES {
                Limb::BYTES
            } else {
                remaining.len()
            };
            let (chunk, tail) = remaining.split_at(chunk_len);
            limbs[limb_idx] = Limb::from_le_slice(chunk);
            remaining = tail;
            limb_idx += 1;
        }
        Ok(Uint::new(limbs))
    }
}

// ── &Uint wrapping ops (needed by ed25519 verify for<'a> &'a T bounds) ───────

impl<const LIMBS: usize> const_num_traits::ops::wrapping::WrappingAdd for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn wrapping_add(self, v: Self) -> Uint<LIMBS> {
        Uint::wrapping_add(self, v)
    }
}

impl<const LIMBS: usize> const_num_traits::ops::wrapping::WrappingSub for &Uint<LIMBS> {
    type Output = Uint<LIMBS>;

    fn wrapping_sub(self, v: Self) -> Uint<LIMBS> {
        Uint::wrapping_sub(self, v)
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
