use std::cmp::Ordering::{self, Less, Greater, Equal};
use std::{u8, u64};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign};
use fixed_sizes::BitLength;
use num::traits::{Zero, One, Unsigned};


/// A `BigDigit` is a `FixedBigUint`'s composing element.
pub type BigDigit = u32;

/// A `DoubleBigDigit` is the internal type used to do the computations.  Its
/// size is the double of the size of `BigDigit`.
pub type DoubleBigDigit = u64;

pub const ZERO_BIG_DIGIT: BigDigit = 0;

#[allow(non_snake_case)]
pub mod big_digit {
    use super::BigDigit;
    use super::DoubleBigDigit;

    // `DoubleBigDigit` size dependent
    pub const BITS: usize = 32;

    pub const BASE: DoubleBigDigit = 1 << BITS;
    const LO_MASK: DoubleBigDigit = (-1i32 as DoubleBigDigit) >> BITS;

    #[inline]
    fn get_hi(n: DoubleBigDigit) -> BigDigit {
        (n >> BITS) as BigDigit
    }
    #[inline]
    fn get_lo(n: DoubleBigDigit) -> BigDigit {
        (n & LO_MASK) as BigDigit
    }

    /// Split one `DoubleBigDigit` into two `BigDigit`s.
    #[inline]
    pub fn from_doublebigdigit(n: DoubleBigDigit) -> (BigDigit, BigDigit) {
        (get_hi(n), get_lo(n))
    }

    /// Join two `BigDigit`s into one `DoubleBigDigit`
    #[inline]
    pub fn to_doublebigdigit(hi: BigDigit, lo: BigDigit) -> DoubleBigDigit {
        (lo as DoubleBigDigit) | ((hi as DoubleBigDigit) << BITS)
    }
}

// Generic functions for add/subtract/multiply with carry/borrow:
//

// Add with carry:
#[inline]
fn adc(a: BigDigit, b: BigDigit, carry: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit((a as DoubleBigDigit) + (b as DoubleBigDigit) +
                                                  (*carry as DoubleBigDigit));

    *carry = hi;
    lo
}

// Subtract with borrow:
#[inline]
fn sbb(a: BigDigit, b: BigDigit, borrow: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit(big_digit::BASE + (a as DoubleBigDigit) -
                                                  (b as DoubleBigDigit) -
                                                  (*borrow as DoubleBigDigit));
    // hi * (base) + lo == 1*(base) + ai - bi - borrow
    // => ai - bi - borrow < 0 <=> hi == 0
    //
    *borrow = if hi == 0 {
        1
    } else {
        0
    };
    lo
}

#[inline]
fn mac_with_carry(a: BigDigit, b: BigDigit, c: BigDigit, carry: &mut BigDigit) -> BigDigit {
    let (hi, lo) = big_digit::from_doublebigdigit((a as DoubleBigDigit) +
                                                  (b as DoubleBigDigit) * (c as DoubleBigDigit) +
                                                  (*carry as DoubleBigDigit));
    *carry = hi;
    lo
}

#[inline]
fn ones_mask(ones: BigDigit) -> BigDigit {
    let ones_count = ones % (big_digit::BITS as BigDigit);
    let mut mask = 0 as BigDigit;

    for i in 0..ones_count {
        mask = (mask << 1) | 1;
    }

    mask
}

/// Divide a two digit numerator by a one digit divisor, returns quotient and remainder:
///
/// Note: the caller must ensure that both the quotient and remainder will fit into a single digit.
/// This is _not_ true for an arbitrary numerator/denominator.
///
/// (This function also matches what the x86 divide instruction does).
#[inline]
fn div_wide(hi: BigDigit, lo: BigDigit, divisor: BigDigit) -> (BigDigit, BigDigit) {
    debug_assert!(hi < divisor);

    let lhs = big_digit::to_doublebigdigit(hi, lo);
    let rhs = divisor as DoubleBigDigit;
    ((lhs / rhs) as BigDigit, (lhs % rhs) as BigDigit)
}

#[derive(Clone, Debug, Hash)]
pub struct FixedBigUint<B> {
    data: Vec<BigDigit>,
    mask: BigDigit,
    size: PhantomData<B>,
}

impl<B> FixedBigUint<B> where B: BitLength {
    /// Creates and initializes a `BigUint`.
    ///
    /// The digits are in little-endian base 2^32. If vector is too long, the 
    /// most significant digits will be truncated.
    #[inline]
    fn new(mut digits: Vec<BigDigit>) -> FixedBigUint<B> {
        let div = B::bit_len() / big_digit::BITS; 
        let rem = B::bit_len() % big_digit::BITS;
        let digit_len = if rem == 0 { div } else { div+1 };
        let mask  = ones_mask(rem as BigDigit);

        if digits.len() > digit_len {
            // If the vector is too long, truncate digits.
            digits.truncate(digit_len);
        }

        // Pad digits to the digit length.
        for i in 0..(digit_len - digits.len()) {
            digits.push(0);
        }

        // digits is not empty here.
        let msd = digits.pop().unwrap() & mask;
        digits.push(msd);

        FixedBigUint {
            data: digits,
            mask: mask,
            size: PhantomData,
        }
    }
}

impl<B> PartialEq for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn eq(&self, other: &FixedBigUint<B>) -> bool {
        match self.cmp(other) {
            Equal => true,
            _     => false,
        }
    }
}

impl<B> Eq for FixedBigUint<B> where B: BitLength {}

impl<B> PartialOrd for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn partial_cmp(&self, other: &FixedBigUint<B>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_slice(a: &[BigDigit], b: &[BigDigit]) -> Ordering {
    debug_assert!(a.last() != Some(&0));
    debug_assert!(b.last() != Some(&0));

    let (a_len, b_len) = (a.len(), b.len());
    if a_len < b_len {
        return Less;
    }
    if a_len > b_len {
        return Greater;
    }

    for (&ai, &bi) in a.iter().rev().zip(b.iter().rev()) {
        if ai < bi {
            return Less;
        }
        if ai > bi {
            return Greater;
        }
    }
    return Equal;
}

impl<B> Ord for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn cmp(&self, other: &FixedBigUint<B>) -> Ordering {
        assert_eq!(self.data.len(), other.data.len());
        assert_eq!(self.size, other.size);

        cmp_slice(&self.data[..], &other.data[..])
    }
}

impl<B> Default for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn default() -> FixedBigUint<B> {
        Zero::zero()
    }
}

impl<B> Zero for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn zero() -> FixedBigUint<B> {
        FixedBigUint::<B>::new(vec![0])
    }

    #[inline]
    fn is_zero(&self) -> bool {
        for term in self.data.iter() {
            if *term != 0 {
                return false;
            }
        }

        true
    }
}
/*
impl<B> One for FixedBigUint<B> where B: BitLength {
    #[inline]
    fn one() -> FixedBigUint<B> {
        let mut uint = Zero::zero();

        uint.data.pop();
        uint.data.push(1 as BigDigit);

        uint
    }
}
*/
//impl<B> Unsigned for FixedBigUint<B> where B: BitLength {}

//forward_all_binop_to_val_ref_commutative!(impl Add for BigUint, add);

// Only for the AddAssign impl:
/// /Two argument addition of raw slices:
/// a += b
///
/// The caller _must_ ensure that a and b are identical in length. Typically this is ensured by
/// using this function to implement addition for fixed precision arithmetic. 
fn __add_assign(a: &mut [BigDigit], b: &[BigDigit], mask: BigDigit) -> BigDigit {
    let mut carry = 0;

    for (ai, bi) in a.iter_mut().zip(b.iter()) {
        if carry != 0 {
            *ai = adc(*ai, 0, &mut carry);
        }

        *ai += adc(*ai, *bi, &mut carry);
    }
    
    carry = (carry & !mask) >> (big_digit::BITS - mask.leading_zeros() as usize);

    carry
}

// Only for the Add impl:
/// /Two argument addition of raw slices:
/// c = a + b
///
/// The caller _must_ ensure that a and b are identical in length. Typically this is ensured by
/// using this function to implement addition for fixed precision arithmetic. 
fn __add<B: BitLength>(a: &[BigDigit], b: &[BigDigit]) -> (FixedBigUint<B>, BigDigit) {
    let mut carry = 0;
    let mut cvec = vec![];

    for (ai, bi) in a.iter().zip(b.iter()) {
        let mut ci = 0; 
        if carry != 0 {
            ci = adc(*ai, 0, &mut carry);
        }

        ci += adc(*ai, *bi, &mut carry);
        cvec.push(ci);
    }

    let c = FixedBigUint::new(cvec);
    carry = (carry & !c.mask) >> (big_digit::BITS - c.mask.leading_zeros() as usize);

    (c, carry)
}

impl<B> Add<FixedBigUint<B>> for FixedBigUint<B> where B: BitLength {
    type Output = FixedBigUint<B>;

    #[allow(unused_variables)]
    fn add(self, other: FixedBigUint<B>) -> FixedBigUint<B> {
        let (result, carry) = __add(&self.data[..], &other.data[..]);

        result
    }
}

impl<'a, B> Add<&'a FixedBigUint<B>> for FixedBigUint<B> where B: BitLength {
    type Output = FixedBigUint<B>;

    #[allow(unused_variables)]
    fn add(self, other: &FixedBigUint<B>) -> FixedBigUint<B> {
        let (result, carry) = __add(&self.data[..], &other.data[..]);

        result
    }
}

impl<'a, B> AddAssign<&'a FixedBigUint<B>> for FixedBigUint<B> where B: BitLength {

    #[allow(unused_variables)]
    fn add_assign(&mut self, other: &'a FixedBigUint<B>) {
        let carry = __add_assign(&mut self.data[..], &other.data[..], self.mask);
    }
}

impl<B> AddAssign<FixedBigUint<B>> for FixedBigUint<B> where B: BitLength {

    #[allow(unused_variables)]
    fn add_assign(&mut self, other: FixedBigUint<B>) {
        let carry = __add_assign(&mut self.data[..], &other.data[..], self.mask);
    }
}
