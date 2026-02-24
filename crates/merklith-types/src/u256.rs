use crate::error::TypesError;
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

/// 256-bit unsigned integer for token amounts and large numbers.
///
/// Stored as 4 x u64 in little-endian limb order.
/// All arithmetic operations check for overflow/underflow and return Result.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct U256([u64; 4]); // [low, mid_low, mid_high, high] little-endian limbs

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for i in (0..4).rev() {
            match self.0[i].cmp(&other.0[i]) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl U256 {
    pub const ZERO: Self = Self([0, 0, 0, 0]);
    pub const ONE: Self = Self([1, 0, 0, 0]);
    pub const MAX: Self = Self([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    /// One MERK in Spark (10^18)
    pub const MERK: Self = Self([1_000_000_000_000_000_000, 0, 0, 0]);

    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(limbs)
    }

    pub const fn as_limbs(&self) -> &[u64; 4] {
        &self.0
    }

    /// Create from a u64 value
    pub const fn from_u64(val: u64) -> Self {
        Self([val, 0, 0, 0])
    }

    /// Create from a u128 value
    pub const fn from_u128(val: u128) -> Self {
        let low = val as u64;
        let high = (val >> 64) as u64;
        Self([low, high, 0, 0])
    }

    /// Checked addition
    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let mut result = [0u64; 4];
        let mut carry = 0u64;

        for i in 0..4 {
            let (sum1, overflow1) = self.0[i].overflowing_add(rhs.0[i]);
            let (sum2, overflow2) = sum1.overflowing_add(carry);
            result[i] = sum2;
            carry = (overflow1 as u64) + (overflow2 as u64);
        }

        if carry != 0 {
            None
        } else {
            Some(Self(result))
        }
    }

    /// Checked subtraction
    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        if self < rhs {
            return None;
        }

        let mut result = [0u64; 4];
        let mut borrow = 0u64;

        for i in 0..4 {
            let (diff1, underflow1) = self.0[i].overflowing_sub(rhs.0[i]);
            let (diff2, underflow2) = diff1.overflowing_sub(borrow);
            result[i] = diff2;
            borrow = (underflow1 as u64) | (underflow2 as u64);
        }

        Some(Self(result))
    }

    /// Checked multiplication
    pub fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        // Check for zero early
        if self.is_zero() || rhs.is_zero() {
            return Some(Self::ZERO);
        }

        let mut result = [0u128; 4];

        for i in 0..4 {
            for j in 0..(4 - i) {
                let product = (self.0[i] as u128) * (rhs.0[j] as u128);
                result[i + j] = result[i + j].checked_add(product)?;
            }
        }

        // Handle carries
        let mut carry = 0u128;
        for i in 0..4 {
            let sum = result[i].checked_add(carry)?;
            result[i] = sum & u64::MAX as u128;
            carry = sum >> 64;
        }

        if carry != 0 {
            return None;
        }

        // Check for overflow in the result
        for i in 0..4 {
            if result[i] > u64::MAX as u128 {
                return None;
            }
        }

        Some(Self([
            result[0] as u64,
            result[1] as u64,
            result[2] as u64,
            result[3] as u64,
        ]))
    }

    /// Checked division
    pub fn checked_div(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // Simple long division
        let mut result = Self::ZERO;
        let mut remainder = Self::ZERO;

        for i in (0..256).rev() {
            remainder = remainder.checked_shl(1)?;
            if self.bit(i) {
                remainder = remainder.checked_add(&Self::ONE)?;
            }

            if remainder >= *rhs {
                remainder = remainder.checked_sub(rhs)?;
                let one_shl_i = Self::ONE.checked_shl(i)?;
                result = result.checked_add(&one_shl_i)?;
            }
        }

        Some(result)
    }

    /// Checked remainder
    pub fn checked_rem(&self, rhs: &Self) -> Option<Self> {
        if rhs.is_zero() {
            return None;
        }

        let div = self.checked_div(rhs)?;
        let mul = div.checked_mul(rhs)?;
        self.checked_sub(&mul)
    }

    /// Checked power
    pub fn checked_pow(&self, exp: u32) -> Option<Self> {
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(*self);
        }

        let mut result = Self::ONE;
        let mut base = *self;
        let mut exp = exp;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.checked_mul(&base)?;
            }
            base = base.checked_mul(&base)?;
            exp >>= 1;
        }

        Some(result)
    }

    /// Saturating addition
    pub fn saturating_add(&self, rhs: &Self) -> Self {
        self.checked_add(rhs).unwrap_or(Self::MAX)
    }

    /// Saturating subtraction
    pub fn saturating_sub(&self, rhs: &Self) -> Self {
        self.checked_sub(rhs).unwrap_or(Self::ZERO)
    }

    /// Saturating multiplication
    pub fn saturating_mul(&self, rhs: &Self) -> Self {
        self.checked_mul(rhs).unwrap_or(Self::MAX)
    }

    /// Bit shift left
    pub fn checked_shl(&self, shift: u32) -> Option<Self> {
        if shift >= 256 {
            return if self.is_zero() {
                Some(Self::ZERO)
            } else {
                None
            };
        }

        let limb_shift = (shift / 64) as usize;
        let bit_shift = shift % 64;

        let mut result = [0u64; 4];

        for i in (limb_shift..4).rev() {
            let src = i - limb_shift;
            result[i] = self.0[src] << bit_shift;

            if bit_shift > 0 && src > 0 {
                result[i] |= self.0[src - 1] >> (64 - bit_shift);
            }
        }

        // Check for overflow
        for i in 0..limb_shift {
            if self.0[4 - limb_shift + i] != 0 {
                return None;
            }
        }

        Some(Self(result))
    }

    /// Bit shift right
    pub fn checked_shr(&self, shift: u32) -> Option<Self> {
        if shift >= 256 {
            return Some(Self::ZERO);
        }

        let limb_shift = (shift / 64) as usize;
        let bit_shift = shift % 64;

        let mut result = [0u64; 4];

        for i in 0..(4 - limb_shift) {
            let src = i + limb_shift;
            result[i] = self.0[src] >> bit_shift;

            if bit_shift > 0 && src + 1 < 4 {
                result[i] |= self.0[src + 1] << (64 - bit_shift);
            }
        }

        Some(Self(result))
    }

    /// Get bit at position
    pub fn bit(&self, pos: u32) -> bool {
        if pos >= 256 {
            return false;
        }
        let limb = (pos / 64) as usize;
        let bit = pos % 64;
        (self.0[limb] >> bit) & 1 != 0
    }

    /// Bit length (position of highest set bit + 1)
    pub fn bit_len(&self) -> u32 {
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                return (i as u32 + 1) * 64 - self.0[i].leading_zeros();
            }
        }
        0
    }

    /// Leading zeros count
    pub fn leading_zeros(&self) -> u32 {
        256 - self.bit_len()
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&l| l == 0)
    }

    /// Integer square root
    pub fn isqrt(&self) -> Self {
        if self.is_zero() {
            return Self::ZERO;
        }

        // Newton-Raphson method
        let mut x = *self;
        let mut y = (x + Self::ONE) >> 1;

        while y < x {
            x = y;
            y = (x + self.checked_div(&x).unwrap_or(Self::ZERO)) >> 1;
        }

        x
    }

    /// Integer log2
    pub fn ilog2(&self) -> u32 {
        self.bit_len().saturating_sub(1)
    }

    /// Convert to big-endian bytes
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let limb_bytes = self.0[3 - i].to_be_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }

    /// Convert from big-endian bytes
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let mut limb_bytes = [0u8; 8];
            limb_bytes.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            limbs[3 - i] = u64::from_be_bytes(limb_bytes);
        }
        Self(limbs)
    }

    /// Convert to little-endian bytes
    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let limb_bytes = self.0[i].to_le_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }

    /// Convert from little-endian bytes
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let mut limb_bytes = [0u8; 8];
            limb_bytes.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            limbs[i] = u64::from_le_bytes(limb_bytes);
        }
        Self(limbs)
    }

    /// Parse from decimal string
    pub fn from_decimal_str(s: &str) -> Result<Self, TypesError> {
        let mut result = Self::ZERO;

        for c in s.chars() {
            if !c.is_ascii_digit() {
                return Err(TypesError::InvalidU256String(s.to_string()));
            }

            let digit = c as u64 - '0' as u64;
            result = result
                .checked_mul(&Self::from_u64(10))
                .ok_or(TypesError::U256Overflow)?;
            result = result
                .checked_add(&Self::from_u64(digit))
                .ok_or(TypesError::U256Overflow)?;
        }

        Ok(result)
    }

    /// Convert to f64 (lossy)
    pub fn to_f64_lossy(&self) -> f64 {
        let mut result = 0.0f64;
        for i in (0..4).rev() {
            result = result.mul_add(2f64.powi(64), self.0[i] as f64);
        }
        result
    }
}

impl From<u64> for U256 {
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u128> for U256 {
    fn from(val: u128) -> Self {
        Self::from_u128(val)
    }
}

impl From<u8> for U256 {
    fn from(val: u8) -> Self {
        Self::from_u64(val as u64)
    }
}

impl U256 {
    /// Raise to a power (returns MAX on overflow - use `checked_pow` for safe version)
    pub fn pow(self, exp: u32) -> Self {
        self.checked_pow(exp).unwrap_or(Self::MAX)
    }

    /// Convert to u128 (returns 0 if value doesn't fit)
    pub fn as_u128(&self) -> u128 {
        (*self).try_into().unwrap_or(0)
    }
}

impl TryFrom<U256> for u64 {
    type Error = TypesError;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        if value.0[1] != 0 || value.0[2] != 0 || value.0[3] != 0 {
            Err(TypesError::U256Overflow)
        } else {
            Ok(value.0[0])
        }
    }
}

impl TryFrom<U256> for u128 {
    type Error = TypesError;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        if value.0[2] != 0 || value.0[3] != 0 {
            Err(TypesError::U256Overflow)
        } else {
            Ok((value.0[1] as u128) << 64 | value.0[0] as u128)
        }
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        let mut n = *self;
        let mut s = String::new();

        while !n.is_zero() {
            let rem = n.checked_rem(&Self::from_u64(10)).map(|v| v.0[0]).unwrap_or(0);
            s.push((rem as u8 + b'0') as char);
            n = n.checked_div(&Self::from_u64(10)).unwrap_or(Self::ZERO);
        }

        write!(f, "{}", s.chars().rev().collect::<String>())
    }
}

impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "U256({})", self)
    }
}

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.to_be_bytes()))
    }
}

impl fmt::UpperHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode_upper(self.to_be_bytes()))
    }
}

impl FromStr for U256 {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") || s.starts_with("0X") {
            let bytes = hex::decode(&s[2..])?;
            if bytes.len() > 32 {
                return Err(TypesError::U256Overflow);
            }
            let mut padded = [0u8; 32];
            padded[32 - bytes.len()..].copy_from_slice(&bytes);
            Ok(Self::from_be_bytes(padded))
        } else {
            Self::from_decimal_str(s)
        }
    }
}

impl Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(&rhs)
    }
}

impl Sub for U256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Saturating subtraction - returns 0 if underflow
        self.checked_sub(&rhs).unwrap_or(Self::ZERO)
    }
}

impl Mul for U256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.saturating_mul(&rhs)
    }
}

impl Mul<U256> for &U256 {
    type Output = U256;

    fn mul(self, rhs: U256) -> Self::Output {
        self.saturating_mul(&rhs)
    }
}

impl Div for U256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(&rhs).unwrap_or(Self::ZERO)
    }
}

impl Rem for U256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.checked_rem(&rhs).unwrap_or(Self::ZERO)
    }
}

impl std::ops::Shr<u32> for U256 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or(Self::ZERO)
    }
}

impl std::ops::Shl<u32> for U256 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs).unwrap_or(Self::ZERO)
    }
}

impl std::ops::AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(&rhs);
    }
}

impl std::ops::SubAssign for U256 {
    fn sub_assign(&mut self, rhs: Self) {
        // Saturating subtraction - clamps to zero
        *self = self.checked_sub(&rhs).unwrap_or(Self::ZERO);
    }
}

impl From<u16> for U256 {
    fn from(val: u16) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<i32> for U256 {
    fn from(val: i32) -> Self {
        assert!(val >= 0, "cannot convert negative i32 to U256");
        Self::from_u64(val as u64)
    }
}

// Bitwise operations
impl std::ops::BitAnd for U256 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl std::ops::BitOr for U256 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

impl std::ops::BitXor for U256 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
            self.0[3] ^ rhs.0[3],
        ])
    }
}

impl std::ops::Not for U256 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self([
            !self.0[0],
            !self.0[1],
            !self.0[2],
            !self.0[3],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_zero_one_max() {
        assert_eq!(U256::ZERO, U256([0, 0, 0, 0]));
        assert_eq!(U256::ONE, U256([1, 0, 0, 0]));
        assert_eq!(U256::MAX, U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX]));
    }

    #[test]
    fn test_u256_from_u64() {
        assert_eq!(U256::from(100u64), U256([100, 0, 0, 0]));
        assert_eq!(U256::from(u64::MAX), U256([u64::MAX, 0, 0, 0]));
    }

    #[test]
    fn test_u256_from_u128() {
        let val: u128 = 0x1234567890abcdef_1122334455667788;
        let u256 = U256::from(val);
        assert_eq!(u256.0[0], 0x1122334455667788);
        assert_eq!(u256.0[1], 0x1234567890abcdef);
        assert_eq!(u256.0[2], 0);
        assert_eq!(u256.0[3], 0);
    }

    #[test]
    fn test_u256_add_no_overflow() {
        let a = U256::from(100u64);
        let b = U256::from(200u64);
        assert_eq!(a.checked_add(&b).unwrap(), U256::from(300u64));
    }

    #[test]
    fn test_u256_add_overflow() {
        let a = U256::MAX;
        let b = U256::ONE;
        assert!(a.checked_add(&b).is_none());
    }

    #[test]
    fn test_u256_sub_no_underflow() {
        let a = U256::from(300u64);
        let b = U256::from(200u64);
        assert_eq!(a.checked_sub(&b).unwrap(), U256::from(100u64));
    }

    #[test]
    fn test_u256_sub_underflow() {
        let a = U256::from(100u64);
        let b = U256::from(200u64);
        assert!(a.checked_sub(&b).is_none());
    }

    #[test]
    fn test_u256_mul() {
        let a = U256::from(100u64);
        let b = U256::from(200u64);
        assert_eq!(a.checked_mul(&b).unwrap(), U256::from(20000u64));
    }

    #[test]
    fn test_u256_div() {
        let a = U256::from(200u64);
        let b = U256::from(10u64);
        assert_eq!(a.checked_div(&b).unwrap(), U256::from(20u64));
    }

    #[test]
    fn test_u256_div_by_zero() {
        let a = U256::from(100u64);
        let b = U256::ZERO;
        assert!(a.checked_div(&b).is_none());
    }

    #[test]
    fn test_u256_isqrt() {
        assert_eq!(U256::from(100u64).isqrt(), U256::from(10u64));
        assert_eq!(U256::from(16u64).isqrt(), U256::from(4u64));
        assert_eq!(U256::from(15u64).isqrt(), U256::from(3u64));
        assert_eq!(U256::ZERO.isqrt(), U256::ZERO);
    }

    #[test]
    fn test_u256_ilog2() {
        assert_eq!(U256::from(1u64).ilog2(), 0);
        assert_eq!(U256::from(2u64).ilog2(), 1);
        assert_eq!(U256::from(4u64).ilog2(), 2);
        assert_eq!(U256::from(8u64).ilog2(), 3);
        assert_eq!(U256::from(255u64).ilog2(), 7);
        assert_eq!(U256::from(256u64).ilog2(), 8);
    }

    #[test]
    fn test_u256_bytes_roundtrip() {
        let original = U256::from(0x1234567890abcdef_1122334455667788u128);
        let be_bytes = original.to_be_bytes();
        let le_bytes = original.to_le_bytes();

        assert_eq!(U256::from_be_bytes(be_bytes), original);
        assert_eq!(U256::from_le_bytes(le_bytes), original);
    }

    #[test]
    fn test_u256_decimal_display() {
        assert_eq!(format!("{}", U256::ZERO), "0");
        assert_eq!(format!("{}", U256::from(12345u64)), "12345");
    }

    #[test]
    fn test_u256_from_decimal_str() {
        assert_eq!(U256::from_str("0").unwrap(), U256::ZERO);
        assert_eq!(U256::from_str("12345").unwrap(), U256::from(12345u64));
        assert_eq!(U256::from_str("0x00").unwrap(), U256::ZERO);
        assert_eq!(U256::from_str("0xFF").unwrap(), U256::from(255u64));
    }

    #[test]
    fn test_u256_ordering() {
        assert!(U256::from(100u64) > U256::from(50u64));
        assert!(U256::from(50u64) < U256::from(100u64));
        assert_eq!(U256::from(100u64), U256::from(100u64));
    }

    #[test]
    fn test_u256_merk() {
        assert_eq!(U256::MERK, U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_u256_bitwise_and() {
        let a = U256::from(0b1010u64);
        let b = U256::from(0b1100u64);
        assert_eq!(a & b, U256::from(0b1000u64));
    }

    #[test]
    fn test_u256_bitwise_or() {
        let a = U256::from(0b1010u64);
        let b = U256::from(0b1100u64);
        assert_eq!(a | b, U256::from(0b1110u64));
    }

    #[test]
    fn test_u256_bitwise_xor() {
        let a = U256::from(0b1010u64);
        let b = U256::from(0b1100u64);
        assert_eq!(a ^ b, U256::from(0b0110u64));
    }

    #[test]
    fn test_u256_bitwise_not() {
        let a = U256::from(0u64);
        assert_eq!(!a, U256::MAX);
        
        let b = U256::MAX;
        assert_eq!(!b, U256::ZERO);
    }

    #[test]
    fn test_u256_pow_checked() {
        let a = U256::from(2u64);
        assert_eq!(a.pow(8), U256::from(256u64));
        
        // Test overflow detection - U256::MAX squared should overflow
        assert!(U256::MAX.checked_pow(2).is_none());
    }
}
