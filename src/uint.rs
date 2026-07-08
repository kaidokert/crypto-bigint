//! Stack-allocated big unsigned integers.

#[cfg(feature = "extra-sizes")]
pub use extra_sizes::*;

pub(crate) use ref_type::UintRef;

use crate::{
    Bounded, Choice, ConstOne, ConstZero, Constants, CtEq, CtOption, EncodedUint, FixedInteger,
    Int, Integer, Limb, NonZero, Odd, One, Unsigned, UnsignedWithMontyForm, Word, Zero, bitlen,
    limb::nlimbs, modular::FixedMontyForm, primitives, traits::sealed::Sealed,
};
use core::fmt;

#[cfg(feature = "serde")]
use crate::Encoding;
#[cfg(feature = "serde")]
use serdect::serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "zeroize")]
use zeroize::DefaultIsZeroes;

#[cfg(doc)]
use crate::{NonZeroUint, OddUint};

#[macro_use]
mod macros;

mod add;
mod add_mod;
mod bit_and;
mod bit_not;
mod bit_or;
mod bit_xor;
mod bits;
mod cmp;
mod concat;
mod ct;
mod div;
pub(crate) mod div_limb;
pub(crate) mod encoding;
mod from;
pub(crate) mod gcd;
mod invert_mod;
pub(crate) mod lcm;
mod mod_symbol;
pub(crate) mod mul;
mod mul_mod;
mod mul_signed;
mod neg;
mod neg_mod;
mod pow;
pub(crate) mod ref_type;
mod resize;
mod root;
mod shl;
mod shr;
mod split;
mod sqrt;
mod sub;
mod sub_mod;

#[cfg(feature = "hybrid-array")]
mod array;
#[cfg(feature = "alloc")]
pub(crate) mod boxed;
#[cfg(feature = "extra-sizes")]
mod extra_sizes;
#[cfg(feature = "cios")]
mod cios;
#[cfg(feature = "rand_core")]
mod rand;

/// Stack-allocated big unsigned integer.
///
/// Generic over the given number of `LIMBS`
///
/// # Encoding support
/// This type supports many different types of encodings, either via the
/// [`Encoding`][`crate::Encoding`] trait or various `const fn` decoding and
/// encoding functions that can be used with [`Uint`] constants.
///
/// Optional crate features for encoding (off-by-default):
/// - `hybrid-array`: enables [`ArrayEncoding`][`crate::ArrayEncoding`] trait which can be used to
///   [`Uint`] as `Array<u8, N>` and a [`ArrayDecoding`][`crate::ArrayDecoding`] trait which
///   can be used to `Array<u8, N>` as [`Uint`].
/// - `rlp`: support for [Recursive Length Prefix (RLP)][RLP] encoding.
///
/// [RLP]: https://eth.wiki/fundamentals/rlp
// TODO(tarcieri): make generic around a specified number of bits.
// Our PartialEq impl only differs from the default one by being constant-time, so this is safe
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Copy, Clone, Hash)]
pub struct Uint<const LIMBS: usize> {
    /// Inner limb array. Stored from least significant to most significant.
    pub(crate) limbs: [Limb; LIMBS],
}

impl<const LIMBS: usize> Uint<LIMBS> {
    /// The value `0`.
    pub const ZERO: Self = Self::from_u8(0);

    /// The value `1`.
    pub const ONE: Self = Self::from_u8(1);

    /// Maximum value this [`Uint`] can express.
    pub const MAX: Self = Self {
        limbs: [Limb::MAX; LIMBS],
    };

    /// Total size of the represented integer in bits.
    pub const BITS: u32 = bitlen::from_limbs(LIMBS);

    /// `floor(log2(Self::BITS))`.
    pub(crate) const LOG2_BITS: u32 = primitives::u32_bits(Self::BITS) - 1;

    /// Total size of the represented integer in bytes.
    pub const BYTES: usize = LIMBS * Limb::BYTES;

    /// The number of limbs used on this platform.
    pub const LIMBS: usize = LIMBS;

    /// Const-friendly [`Uint`] constructor.
    #[must_use]
    pub const fn new(limbs: [Limb; LIMBS]) -> Self {
        Self { limbs }
    }

    /// Create a [`Uint`] from an array of [`Word`]s (i.e. word-sized unsigned
    /// integers).
    #[inline]
    #[must_use]
    pub const fn from_words(arr: [Word; LIMBS]) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            limbs[i] = Limb(arr[i]);
            i += 1;
        }

        Self { limbs }
    }

    /// Create an array of [`Word`]s (i.e. word-sized unsigned integers) from
    /// a [`Uint`].
    #[inline]
    #[must_use]
    pub const fn to_words(self) -> [Word; LIMBS] {
        let mut arr = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            arr[i] = self.limbs[i].0;
            i += 1;
        }

        arr
    }

    /// Borrow the inner limbs as an array of [`Word`]s.
    #[must_use]
    pub const fn as_words(&self) -> &[Word; LIMBS] {
        Limb::array_as_words(&self.limbs)
    }

    /// Borrow the inner limbs as a mutable array of [`Word`]s.
    pub const fn as_mut_words(&mut self) -> &mut [Word; LIMBS] {
        Limb::array_as_mut_words(&mut self.limbs)
    }

    /// Borrow the inner limbs as a mutable slice of [`Word`]s.
    #[deprecated(since = "0.7.0", note = "please use `as_mut_words` instead")]
    pub const fn as_words_mut(&mut self) -> &mut [Word] {
        self.as_mut_words()
    }

    /// Borrow the limbs of this [`Uint`].
    #[must_use]
    pub const fn as_limbs(&self) -> &[Limb; LIMBS] {
        &self.limbs
    }

    /// Borrow the limbs of this [`Uint`] mutably.
    pub const fn as_mut_limbs(&mut self) -> &mut [Limb; LIMBS] {
        &mut self.limbs
    }

    /// Borrow the limbs of this [`Uint`] mutably.
    #[deprecated(since = "0.7.0", note = "please use `as_mut_limbs` instead")]
    pub const fn as_limbs_mut(&mut self) -> &mut [Limb] {
        self.as_mut_limbs()
    }

    /// Convert this [`Uint`] into its inner limbs.
    #[must_use]
    pub const fn to_limbs(self) -> [Limb; LIMBS] {
        self.limbs
    }

    /// Borrow the limbs of this [`Uint`] as a [`UintRef`].
    #[inline]
    #[must_use]
    pub const fn as_uint_ref(&self) -> &UintRef {
        UintRef::new(&self.limbs)
    }

    /// Mutably borrow the limbs of this [`Uint`] as a [`UintRef`].
    #[inline]
    #[must_use]
    pub const fn as_mut_uint_ref(&mut self) -> &mut UintRef {
        UintRef::new_mut(&mut self.limbs)
    }

    /// Construct a [`NonZero`] reference, returning [`None`] in the event `self` is `0`.
    #[inline]
    #[must_use]
    pub const fn as_nz_vartime(&self) -> Option<&NonZero<Self>> {
        if self.is_zero_vartime() {
            None
        } else {
            Some(NonZero::new_ref_unchecked(self))
        }
    }

    /// Convert to a [`NonZeroUint<LIMBS>`].
    ///
    /// Returns some if the original value is non-zero, and none otherwise.
    #[must_use]
    pub const fn to_nz(&self) -> CtOption<NonZero<Self>> {
        let (nz, self_nz) = self.to_nz_or_one();
        CtOption::new(nz, self_nz)
    }

    /// Convert to a [`NonZeroUint<LIMBS>`].
    ///
    /// Returns Some if the original value is non-zero, and none otherwise.
    #[must_use]
    pub const fn to_nz_vartime(&self) -> Option<NonZero<Self>> {
        if self.is_zero_vartime() {
            None
        } else {
            Some(NonZero::new_unchecked(*self))
        }
    }

    /// Convert to a [`NonZeroUint<LIMBS>`], defaulting to `Self::ONE`.
    ///
    /// Returns a pair consisting of a [`NonZeroUint<LIMBS>`], and a [`Choice`]
    /// indicating whether the original value was non-zero (and preserved).
    #[inline(always)]
    #[must_use]
    pub(crate) const fn to_nz_or_one(self) -> (NonZero<Self>, Choice) {
        let is_nz = self.is_nonzero();
        (
            NonZero::new_unchecked(Self::select(&Self::ONE, &self, is_nz)),
            is_nz,
        )
    }

    /// Convert to a [`OddUint<LIMBS>`].
    ///
    /// Returns some if the original value is odd, and none otherwise.
    #[must_use]
    pub const fn to_odd(&self) -> CtOption<Odd<Self>> {
        let (odd, self_odd) = self.to_odd_or_one();
        CtOption::new(odd, self_odd)
    }

    /// Convert to a [`OddUint<LIMBS>`], defaulting to `Self::ONE`.
    ///
    /// Returns a pair consisting of a [`OddUint<LIMBS>`], and a [`Choice`]
    /// indicating whether the original value was non-zero (and preserved).
    #[inline(always)]
    #[must_use]
    pub(crate) const fn to_odd_or_one(self) -> (Odd<Self>, Choice) {
        let is_odd = self.is_odd();
        (
            Odd::new_unchecked(Self::select(&Self::ONE, &self, is_odd)),
            is_odd,
        )
    }

    /// Interpret this object as an [`Int`] instead.
    ///
    /// Note: this is a casting operation. See [`Self::try_into_int`] for the checked equivalent.
    #[must_use]
    pub const fn as_int(&self) -> &Int<LIMBS> {
        // SAFETY: `Int` is a `repr(transparent)` newtype for `Uint`, and this operation is intended
        // to be a reinterpreting cast between the two types.
        #[allow(unsafe_code)]
        unsafe {
            &*core::ptr::from_ref(self).cast::<Int<LIMBS>>()
        }
    }

    /// Convert this type into an [`Int`]; returns `None` if this value is greater than [`Int::MAX`].
    ///
    /// Note: this is the conversion operation. See [`Self::as_int`] for the unchecked equivalent.
    #[must_use]
    pub const fn try_into_int(self) -> CtOption<Int<LIMBS>> {
        Int::new_from_abs_sign(self, Choice::FALSE)
    }

    /// Is this [`Uint`] equal to [`Uint::ZERO`]?
    #[must_use]
    pub const fn is_zero(&self) -> Choice {
        self.is_nonzero().not()
    }
}

impl<const LIMBS: usize> AsRef<[Word; LIMBS]> for Uint<LIMBS> {
    fn as_ref(&self) -> &[Word; LIMBS] {
        self.as_words()
    }
}

impl<const LIMBS: usize> AsMut<[Word; LIMBS]> for Uint<LIMBS> {
    fn as_mut(&mut self) -> &mut [Word; LIMBS] {
        self.as_mut_words()
    }
}

impl<const LIMBS: usize> AsRef<[Limb]> for Uint<LIMBS> {
    fn as_ref(&self) -> &[Limb] {
        self.as_limbs()
    }
}

impl<const LIMBS: usize> AsMut<[Limb]> for Uint<LIMBS> {
    fn as_mut(&mut self) -> &mut [Limb] {
        self.as_mut_limbs()
    }
}

impl<const LIMBS: usize> AsRef<UintRef> for Uint<LIMBS> {
    fn as_ref(&self) -> &UintRef {
        self.as_uint_ref()
    }
}

impl<const LIMBS: usize> AsMut<UintRef> for Uint<LIMBS> {
    fn as_mut(&mut self) -> &mut UintRef {
        self.as_mut_uint_ref()
    }
}

impl<const LIMBS: usize> Bounded for Uint<LIMBS> {
    const BITS: u32 = Self::BITS;
    const BYTES: usize = Self::BYTES;
}

impl<const LIMBS: usize> Constants for Uint<LIMBS> {
    const MAX: Self = Self::MAX;
}

impl<const LIMBS: usize> Default for Uint<LIMBS> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const LIMBS: usize> FixedInteger for Uint<LIMBS> {
    const LIMBS: usize = LIMBS;
}

impl<const LIMBS: usize> Integer for Uint<LIMBS> {
    fn as_limbs(&self) -> &[Limb] {
        &self.limbs
    }

    fn as_mut_limbs(&mut self) -> &mut [Limb] {
        &mut self.limbs
    }

    fn nlimbs(&self) -> usize {
        Self::LIMBS
    }
}

impl<const LIMBS: usize> Sealed for Uint<LIMBS> {}

impl<const LIMBS: usize> Unsigned for Uint<LIMBS> {
    fn as_uint_ref(&self) -> &UintRef {
        self.as_uint_ref()
    }

    fn as_mut_uint_ref(&mut self) -> &mut UintRef {
        self.as_mut_uint_ref()
    }

    fn from_limb_like(limb: Limb, _other: &Self) -> Self {
        Self::from(limb)
    }
}

impl<const LIMBS: usize> UnsignedWithMontyForm for Uint<LIMBS> {
    type MontyForm = FixedMontyForm<LIMBS>;
}

impl<const LIMBS: usize> num_traits::Num for Uint<LIMBS> {
    type FromStrRadixErr = crate::DecodeError;

    /// <div class="warning">
    /// <b>WARNING: variable-time!</b>
    ///
    /// `from_str_radix` impl operates in variable-time with respect to the input.
    /// </div>
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Self::from_str_radix_vartime(str, radix)
    }
}

impl<const LIMBS: usize> ConstZero for Uint<LIMBS> {
    const ZERO: Self = Self::ZERO;
}

impl<const LIMBS: usize> ConstOne for Uint<LIMBS> {
    const ONE: Self = Self::ONE;
}

impl<const LIMBS: usize> Zero for Uint<LIMBS> {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }
}

impl<const LIMBS: usize> One for Uint<LIMBS> {
    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }
}

impl<const LIMBS: usize> num_traits::Zero for Uint<LIMBS> {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.ct_eq(&Self::ZERO).into()
    }

    fn set_zero(&mut self) {
        *self = Self::ZERO;
    }
}

impl<const LIMBS: usize> num_traits::One for Uint<LIMBS> {
    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    fn is_one(&self) -> bool {
        self.ct_eq(&Self::ONE).into()
    }

    fn set_one(&mut self) {
        *self = Self::ONE;
    }
}

impl<const LIMBS: usize> fmt::Debug for Uint<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uint(0x{:X})", self.as_uint_ref())
    }
}

impl<const LIMBS: usize> fmt::Binary for Uint<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(self.as_uint_ref(), f)
    }
}

impl<const LIMBS: usize> fmt::Display for Uint<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}

impl<const LIMBS: usize> fmt::LowerHex for Uint<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(self.as_uint_ref(), f)
    }
}

impl<const LIMBS: usize> fmt::UpperHex for Uint<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self.as_uint_ref(), f)
    }
}

#[cfg(feature = "serde")]
impl<'de, const LIMBS: usize> Deserialize<'de> for Uint<LIMBS>
where
    Uint<LIMBS>: Encoding,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut buffer = Encoding::to_le_bytes(&Self::ZERO);
        serdect::array::deserialize_hex_or_bin(buffer.as_mut(), deserializer)?;

        Ok(Encoding::from_le_bytes(buffer))
    }
}

#[cfg(feature = "serde")]
impl<const LIMBS: usize> Serialize for Uint<LIMBS>
where
    Uint<LIMBS>: Encoding,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serdect::slice::serialize_hex_lower_or_bin(&Encoding::to_le_bytes(self), serializer)
    }
}

#[cfg(feature = "zeroize")]
impl<const LIMBS: usize> DefaultIsZeroes for Uint<LIMBS> {}

impl<const LIMBS: usize> num_traits::Bounded for Uint<LIMBS> {
    fn min_value() -> Self { Self::ZERO }
    fn max_value() -> Self { Self::MAX }
}

impl<const LIMBS: usize> num_traits::CheckedAdd for Uint<LIMBS> {
    fn checked_add(self, rhs: Self) -> Option<Self> {
        let (result, carry) = self.carrying_add(&rhs, Limb::ZERO);
        if carry.eq_vartime(Limb::ZERO) { Some(result) } else { None }
    }
}

impl<const LIMBS: usize> num_traits::CheckedSub for Uint<LIMBS> {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (result, borrow) = self.sbb(&rhs, Limb::ZERO);
        if borrow.eq_vartime(Limb::ZERO) { Some(result) } else { None }
    }
}

impl<const LIMBS: usize> num_traits::Saturating for Uint<LIMBS> {
    fn saturating_add(self, rhs: Self) -> Self {
        Uint::saturating_add(&self, &rhs)
    }

    fn saturating_sub(self, rhs: Self) -> Self {
        Uint::saturating_sub(&self, &rhs)
    }
}

impl<const LIMBS: usize> num_traits::CheckedMul for Uint<LIMBS> {
    fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (lo, hi) = self.widening_mul::<LIMBS>(&rhs);
        if hi.is_zero_vartime() { Some(lo) } else { None }
    }
}

impl<const LIMBS: usize> num_traits::CheckedDiv for Uint<LIMBS> {
    fn checked_div(self, rhs: Self) -> Option<Self> {
        NonZero::new(rhs).map(|nz| self.wrapping_div(&nz)).into()
    }
}

impl<const LIMBS: usize> num_traits::ToPrimitive for Uint<LIMBS> {
    fn to_i64(&self) -> Option<i64> {
        let u = self.to_u64()?;
        if u <= i64::MAX as u64 { Some(u as i64) } else { None }
    }

    fn to_u64(&self) -> Option<u64> {
        if *self > Self::from_u64(u64::MAX) { return None; }
        Some(self.as_words()[0])
    }
}

impl<const LIMBS: usize> num_traits::NumCast for Uint<LIMBS> {
    fn from<T: num_traits::ToPrimitive>(n: T) -> Option<Self> {
        n.to_u64().map(Self::from_u64)
    }
}

// const-num-traits splits bit methods out of `PrimInt` into `PrimBits`.
impl<const LIMBS: usize> num_traits::OverflowingAdd for Uint<LIMBS> {
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (result, carry) = self.carrying_add(&rhs, Limb::ZERO);
        (result, bool::from(carry.is_nonzero()))
    }
}

impl<const LIMBS: usize> num_traits::OverflowingSub for Uint<LIMBS> {
    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let (result, borrow) = self.sbb(&rhs, Limb::ZERO);
        (result, bool::from(borrow.is_nonzero()))
    }
}

impl<const LIMBS: usize> num_traits::OverflowingMul for Uint<LIMBS> {
    fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let (lo, hi) = self.widening_mul::<LIMBS>(&rhs);
        (lo, !hi.is_zero_vartime())
    }
}

impl<const LIMBS: usize> num_traits::PrimBits for Uint<LIMBS> {
    fn count_ones(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS { n += self.limbs[i].0.count_ones(); i += 1; }
        n
    }

    fn count_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS { n += self.limbs[i].0.count_zeros(); i += 1; }
        n
    }

    fn leading_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = LIMBS;
        while i > 0 {
            i -= 1;
            let lz = self.limbs[i].0.leading_zeros();
            n += lz;
            if lz < Limb::BITS { break; }
        }
        n
    }

    fn trailing_zeros(self) -> u32 {
        let mut n = 0u32;
        let mut i = 0;
        while i < LIMBS {
            let tz = self.limbs[i].0.trailing_zeros();
            n += tz;
            if tz < Limb::BITS { break; }
            i += 1;
        }
        n
    }

    fn rotate_left(self, _n: u32) -> Self { todo!() }
    fn rotate_right(self, _n: u32) -> Self { todo!() }
    fn signed_shl(self, _n: u32) -> Self { todo!() }
    fn signed_shr(self, _n: u32) -> Self { todo!() }
    fn unsigned_shl(self, n: u32) -> Self { self.wrapping_shl(n) }
    fn unsigned_shr(self, n: u32) -> Self { self.wrapping_shr(n) }
    fn swap_bytes(self) -> Self { todo!() }
    fn from_be(x: Self) -> Self { todo!() }
    fn from_le(x: Self) -> Self { todo!() }
    fn to_be(self) -> Self { todo!() }
    fn to_le(self) -> Self { todo!() }
}

impl<const LIMBS: usize> num_traits::PrimInt for Uint<LIMBS> {
    fn pow(self, mut exp: u32) -> Self {
        let mut base = self;
        let mut result = Self::ONE;
        while exp > 0 {
            if exp & 1 == 1 { result = Uint::wrapping_mul(&result, &base); }
            base = Uint::wrapping_mul(&base, &base);
            exp >>= 1;
        }
        result
    }
}

// Fixed-size byte buffer: stores [Limb; LIMBS], exposes LIMBS*Limb::BYTES via
// unsafe ptr cast. Default via [Limb::ZERO; LIMBS] works for any LIMBS.
// Unifies ToBytes::Bytes == FromBytes::Bytes for FixedWidthUnsignedInt compat.
#[derive(Clone, Copy)]
pub struct BytesHolder<const LIMBS: usize> {
    limbs: [Limb; LIMBS],
}

impl<const LIMBS: usize> Default for BytesHolder<LIMBS> {
    fn default() -> Self {
        Self { limbs: [Limb::ZERO; LIMBS] }
    }
}

impl<const LIMBS: usize> BytesHolder<LIMBS> {
    #[allow(unsafe_code)]
    #[inline]
    fn as_byte_slice(&self) -> &[u8] {
        // SAFETY: Limb is repr(transparent) over Word; array is valid for
        // reads of LIMBS * Limb::BYTES bytes from its base address.
        unsafe {
            core::slice::from_raw_parts(
                self.limbs.as_ptr() as *const u8,
                LIMBS * Limb::BYTES,
            )
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: unique &mut access; same size guarantee as as_byte_slice.
        unsafe {
            core::slice::from_raw_parts_mut(
                self.limbs.as_mut_ptr() as *mut u8,
                LIMBS * Limb::BYTES,
            )
        }
    }
}

impl<const LIMBS: usize> AsRef<[u8]> for BytesHolder<LIMBS> {
    fn as_ref(&self) -> &[u8] { self.as_byte_slice() }
}
impl<const LIMBS: usize> AsMut<[u8]> for BytesHolder<LIMBS> {
    fn as_mut(&mut self) -> &mut [u8] { self.as_byte_slice_mut() }
}
impl<const LIMBS: usize> core::borrow::Borrow<[u8]> for BytesHolder<LIMBS> {
    fn borrow(&self) -> &[u8] { self.as_byte_slice() }
}
impl<const LIMBS: usize> core::borrow::BorrowMut<[u8]> for BytesHolder<LIMBS> {
    fn borrow_mut(&mut self) -> &mut [u8] { self.as_byte_slice_mut() }
}
impl<const LIMBS: usize> PartialEq for BytesHolder<LIMBS> {
    fn eq(&self, other: &Self) -> bool { self.as_byte_slice() == other.as_byte_slice() }
}
impl<const LIMBS: usize> Eq for BytesHolder<LIMBS> {}
impl<const LIMBS: usize> PartialOrd for BytesHolder<LIMBS> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> { Some(self.cmp(other)) }
}
impl<const LIMBS: usize> Ord for BytesHolder<LIMBS> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering { self.as_byte_slice().cmp(other.as_byte_slice()) }
}
impl<const LIMBS: usize> core::hash::Hash for BytesHolder<LIMBS> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) { self.as_byte_slice().hash(state) }
}
impl<const LIMBS: usize> fmt::Debug for BytesHolder<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BytesHolder(")?;
        for b in self.as_byte_slice() { write!(f, "{:02x}", b)?; }
        write!(f, ")")
    }
}
#[cfg(feature = "zeroize")]
impl<const LIMBS: usize> zeroize::DefaultIsZeroes for BytesHolder<LIMBS> {}

impl<const LIMBS: usize> num_traits::ToBytes for Uint<LIMBS> {
    type Bytes = BytesHolder<LIMBS>;

    fn to_be_bytes(self) -> BytesHolder<LIMBS> {
        let mut holder = BytesHolder::default();
        let bytes = holder.as_byte_slice_mut();
        let limb_bytes = Limb::BYTES;
        let mut i = 0;
        while i < LIMBS {
            let word_be = self.limbs[LIMBS - 1 - i].0.to_be_bytes();
            let mut j = 0;
            while j < limb_bytes { bytes[i * limb_bytes + j] = word_be[j]; j += 1; }
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
            while j < limb_bytes { bytes[i * limb_bytes + j] = word_le[j]; j += 1; }
            i += 1;
        }
        holder
    }
}

impl<const LIMBS: usize> num_traits::FromBytes for Uint<LIMBS> {
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
            while j < limb_bytes { word = (word << 8) | (bytes[start + j] as Word); j += 1; }
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
            while j < limb_bytes { word |= (bytes[start + j] as Word) << (j * 8); j += 1; }
            limbs[i] = Limb(word);
            i += 1;
        }
        Uint::new(limbs)
    }
}

// For the signing path: `for<'a> &'a T: ToBytes<Bytes = <T as ToBytes>::Bytes>`
impl<const LIMBS: usize> num_traits::ToBytes for &Uint<LIMBS> {
    type Bytes = BytesHolder<LIMBS>;
    fn to_be_bytes(self) -> BytesHolder<LIMBS> { (*self).to_be_bytes() }
    fn to_le_bytes(self) -> BytesHolder<LIMBS> { (*self).to_le_bytes() }
}

impl<const LIMBS: usize> num_traits::ops::parity::Parity for Uint<LIMBS> {
    fn is_odd(self) -> bool {
        if LIMBS == 0 { return false; }
        self.limbs[0].0 & 1 == 1
    }
    fn is_even(self) -> bool { !self.is_odd() }
}

impl<const LIMBS: usize> num_traits::ops::carrying::BorrowingSub for Uint<LIMBS> {
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (res, b) = self.sbb(&rhs, Limb(borrow as Word));
        (res, b.0 != 0)
    }
}

impl<const LIMBS: usize> num_traits::ops::carrying::CarryingMul for Uint<LIMBS> {
    type Unsigned = Self;

    fn carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        let (lo, hi) = self.split_mul(&rhs);
        let (lo, c) = lo.adc(&carry, Limb::ZERO);
        let (hi, _) = hi.adc(&Uint::ZERO, c);
        (lo, hi)
    }

    fn carrying_mul_add(self, rhs: Self, carry: Self, add: Self) -> (Self, Self) {
        let (lo, hi) = self.split_mul(&rhs);
        let (lo, c1) = lo.adc(&carry, Limb::ZERO);
        let (lo, c2) = lo.adc(&add, Limb::ZERO);
        let carry_total = Limb(c1.0.wrapping_add(c2.0));
        let (hi, _) = hi.adc(&Uint::ZERO, carry_total);
        (lo, hi)
    }
}

impl<const LIMBS: usize> num_traits::personality::HasPersonality for Uint<LIMBS> {
    type P = num_traits::Nct;
}

impl<const LIMBS: usize> num_traits::ops::byte_slice::FromByteSlice for Uint<LIMBS> {
    fn from_be_slice(bytes: &[u8]) -> Result<Self, num_traits::ops::byte_slice::ByteSliceError> {
        use num_traits::ops::byte_slice::ByteSliceErrorKind;
        if bytes.is_empty() {
            return Err(num_traits::ops::byte_slice::ByteSliceError { kind: ByteSliceErrorKind::Empty });
        }
        let capacity = LIMBS * Limb::BYTES;
        if bytes.len() > capacity {
            return Err(num_traits::ops::byte_slice::ByteSliceError { kind: ByteSliceErrorKind::Overflow });
        }
        // Zero-extend: pad to the left with leading zeros.
        let mut limbs = [Limb::ZERO; LIMBS];
        // Iterate from the right (LSB limb) upward, consuming Limb::BYTES from the end of bytes.
        let mut remaining = bytes;
        let mut limb_idx = 0usize;
        while limb_idx < LIMBS && !remaining.is_empty() {
            let chunk_len = if remaining.len() >= Limb::BYTES { Limb::BYTES } else { remaining.len() };
            let (head, tail) = remaining.split_at(remaining.len() - chunk_len);
            limbs[limb_idx] = Limb::from_be_slice(tail);
            remaining = head;
            limb_idx += 1;
        }
        Ok(Uint::new(limbs))
    }

    fn from_le_slice(bytes: &[u8]) -> Result<Self, num_traits::ops::byte_slice::ByteSliceError> {
        use num_traits::ops::byte_slice::ByteSliceErrorKind;
        if bytes.is_empty() {
            return Err(num_traits::ops::byte_slice::ByteSliceError { kind: ByteSliceErrorKind::Empty });
        }
        let capacity = LIMBS * Limb::BYTES;
        if bytes.len() > capacity {
            return Err(num_traits::ops::byte_slice::ByteSliceError { kind: ByteSliceErrorKind::Overflow });
        }
        // Zero-extend: pad to the right with trailing zeros.
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut remaining = bytes;
        let mut limb_idx = 0usize;
        while limb_idx < LIMBS && !remaining.is_empty() {
            let chunk_len = if remaining.len() >= Limb::BYTES { Limb::BYTES } else { remaining.len() };
            let (chunk, tail) = remaining.split_at(chunk_len);
            limbs[limb_idx] = Limb::from_le_slice(chunk);
            remaining = tail;
            limb_idx += 1;
        }
        Ok(Uint::new(limbs))
    }
}

// TODO(tarcieri): use `generic_const_exprs` when stable to make generic around bits.
impl_uint_aliases! {
    (U64, 64, "64-bit"),
    (U128, 128, "128-bit"),
    (U192, 192, "192-bit"),
    (U256, 256, "256-bit"),
    (U320, 320, "320-bit"),
    (U384, 384, "384-bit"),
    (U448, 448, "448-bit"),
    (U512, 512, "512-bit"),
    (U576, 576, "576-bit"),
    (U640, 640, "640-bit"),
    (U704, 704, "704-bit"),
    (U768, 768, "768-bit"),
    (U832, 832, "832-bit"),
    (U896, 896, "896-bit"),
    (U960, 960, "960-bit"),
    (U1024, 1024, "1024-bit"),
    (U1280, 1280, "1280-bit"),
    (U1536, 1536, "1536-bit"),
    (U1792, 1792, "1792-bit"),
    (U2048, 2048, "2048-bit"),
    (U3072, 3072, "3072-bit"),
    (U3584, 3584, "3584-bit"),
    (U4096, 4096, "4096-bit"),
    (U4224, 4224, "4224-bit"),
    (U4352, 4352, "4352-bit"),
    (U6144, 6144, "6144-bit"),
    (U8192, 8192, "8192-bit"),
    (U16384, 16384, "16384-bit"),
    (U32768, 32768, "32768-bit")
}

cpubits::cpubits! {
    32 => {
        impl_uint_aliases! {
            (U224, 224, "224-bit"), // For NIST P-224
            (U544, 544, "544-bit")  // For NIST P-521
        }
        impl_uint_concat_split_even! {
            U64,
        }
    }
}

// Implement concat and split for double-width Uint sizes: these should be
// multiples of 128 bits.
impl_uint_concat_split_even! {
    U128,
    U256,
    U384,
    U512,
    U640,
    U768,
    U896,
    U1024,
    U1280,
    U1536,
    U1792,
    U2048,
    U3072,
    U3584,
    U4096,
    U4224,
    U4352,
    U6144,
    U8192,
    U16384,
}

// Implement mixed concat, split and reduce for combinations not implemented by
// impl_uint_concat_split_even. The numbers represent the size of each
// component Uint in multiple of 64 bits. For example,
// (U256, [1, 3]) will allow splitting U256 into (U64, U192) as well as
// (U192, U64), while the (U128, U128) combination is already covered.
impl_uint_concat_split_mixed! {
    (U192, [1, 2]),
    (U256, [1, 3]),
    (U320, [1, 2, 3, 4]),
    (U384, [1, 2, 4, 5]),
    (U448, [1, 2, 3, 4, 5, 6]),
    (U512, [1, 2, 3, 5, 6, 7]),
    (U576, [1, 2, 3, 4, 5, 6, 7, 8]),
    (U640, [1, 2, 3, 4, 6, 7, 8, 9]),
    (U704, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    (U768, [1, 2, 3, 4, 5, 7, 8, 9, 10, 11]),
    (U832, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
    (U896, [1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13]),
    (U960, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]),
    (U1024, [1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15]),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{Encoding, I128, Int, U128};

    #[cfg(feature = "alloc")]
    use alloc::format;

    cpubits::cpubits! {
        64 => {
            #[test]
            fn as_words() {
                let n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
                assert_eq!(n.as_words(), &[0xCCCCCCCCDDDDDDDD, 0xAAAAAAAABBBBBBBB]);
            }

            #[test]
            fn as_words_mut() {
                let mut n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
                assert_eq!(n.as_mut_words(), &[0xCCCCCCCCDDDDDDDD, 0xAAAAAAAABBBBBBBB]);
            }
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn debug() {
        let n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");

        assert_eq!(format!("{n:?}"), "Uint(0xAAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD)");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn display() {
        let hex = "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD";
        let n = U128::from_be_hex(hex);

        use alloc::string::ToString;
        assert_eq!(hex, n.to_string());

        let hex = "AAAAAAAABBBBBBBB0000000000000000";
        let n = U128::from_be_hex(hex);
        assert_eq!(hex, n.to_string());

        let hex = "AAAAAAAABBBBBBBB00000000DDDDDDDD";
        let n = U128::from_be_hex(hex);
        assert_eq!(hex, n.to_string());

        let hex = "AAAAAAAABBBBBBBB0CCCCCCCDDDDDDDD";
        let n = U128::from_be_hex(hex);
        assert_eq!(hex, n.to_string());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_lower_hex() {
        let n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
        assert_eq!(format!("{n:x}"), "aaaaaaaabbbbbbbbccccccccdddddddd");
        assert_eq!(format!("{n:#x}"), "0xaaaaaaaabbbbbbbbccccccccdddddddd");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_lower_hex_from_trait() {
        fn format_int<T: crate::Integer>(n: T) -> alloc::string::String {
            format!("{n:x}")
        }
        let n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
        assert_eq!(format_int(n), "aaaaaaaabbbbbbbbccccccccdddddddd");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_upper_hex() {
        let n = U128::from_be_hex("aaaaaaaabbbbbbbbccccccccdddddddd");
        assert_eq!(format!("{n:X}"), "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
        assert_eq!(format!("{n:#X}"), "0xAAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_upper_hex_from_trait() {
        fn format_int<T: crate::Integer>(n: T) -> alloc::string::String {
            format!("{n:X}")
        }
        let n = U128::from_be_hex("aaaaaaaabbbbbbbbccccccccdddddddd");
        assert_eq!(format_int(n), "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_binary() {
        let n = U128::from_be_hex("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD");
        assert_eq!(
            format!("{n:b}"),
            "10101010101010101010101010101010101110111011101110111011101110111100110011001100110011001100110011011101110111011101110111011101"
        );
        assert_eq!(
            format!("{n:#b}"),
            "0b10101010101010101010101010101010101110111011101110111011101110111100110011001100110011001100110011011101110111011101110111011101"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn fmt_binary_from_trait() {
        fn format_int<T: crate::Integer>(n: T) -> alloc::string::String {
            format!("{n:b}")
        }
        let n = U128::from_be_hex("aaaaaaaabbbbbbbbccccccccdddddddd");
        assert_eq!(
            format_int(n),
            "10101010101010101010101010101010101110111011101110111011101110111100110011001100110011001100110011011101110111011101110111011101"
        );
    }

    #[test]
    fn from_bytes() {
        let a = U128::from_be_hex("AAAAAAAABBBBBBBB0CCCCCCCDDDDDDDD");

        let be_bytes = a.to_be_bytes();
        let le_bytes = a.to_le_bytes();
        for i in 0..16 {
            assert_eq!(le_bytes.as_ref()[i], be_bytes.as_ref()[15 - i]);
        }

        let a_from_be = U128::from_be_bytes(be_bytes);
        let a_from_le = U128::from_le_bytes(le_bytes);
        assert_eq!(a_from_be, a_from_le);
        assert_eq!(a_from_be, a);
    }

    #[test]
    fn as_int() {
        assert_eq!(*U128::ZERO.as_int(), Int::ZERO);
        assert_eq!(*U128::ONE.as_int(), Int::ONE);
        assert_eq!(*U128::MAX.as_int(), Int::MINUS_ONE);
    }

    #[test]
    fn to_int() {
        assert_eq!(U128::ZERO.try_into_int().unwrap(), Int::ZERO);
        assert_eq!(U128::ONE.try_into_int().unwrap(), Int::ONE);
        assert_eq!(I128::MAX.as_uint().try_into_int().unwrap(), Int::MAX);
        assert!(bool::from(U128::MAX.try_into_int().is_none()));
    }

    #[test]
    fn test_unsigned() {
        crate::traits::tests::test_unsigned(U128::ZERO, U128::MAX);
    }

    #[test]
    fn test_unsigned_monty_form() {
        crate::traits::tests::test_unsigned_monty_form::<U128>();
    }
}
