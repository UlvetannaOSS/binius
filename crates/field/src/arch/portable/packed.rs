// Copyright 2024 Ulvetanna Inc.

// This is because derive(bytemuck::TransparentWrapper) adds some type constraints to
// PackedPrimitiveType in addition to the type constraints we define. Even more, annoying, the
// allow attribute has to be added to the module, it doesn't work to add it to the struct
// definition.
#![allow(clippy::multiple_bound_locations)]

use super::packed_arithmetic::UnderlierWithBitConstants;
use crate::{
	arithmetic_traits::{Broadcast, InvertOrZero, MulAlpha, Square},
	underlier::{NumCast, UnderlierType, UnderlierWithBitOps, WithUnderlier, U1, U2, U4},
	BinaryField, Error, PackedField,
};
use bytemuck::{Pod, TransparentWrapper, Zeroable};
use rand::RngCore;
use std::{
	fmt::Debug,
	iter::{Product, Sum},
	marker::PhantomData,
	ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};
use subtle::{Choice, ConstantTimeEq};

#[derive(PartialEq, Eq, Clone, Copy, Default, bytemuck::TransparentWrapper)]
#[repr(transparent)]
#[transparent(U)]
pub struct PackedPrimitiveType<U: UnderlierType, Scalar: BinaryField>(
	pub U,
	pub PhantomData<Scalar>,
);

impl<U: UnderlierType, Scalar: BinaryField> PackedPrimitiveType<U, Scalar> {
	pub const WIDTH: usize = {
		assert!(U::BITS % Scalar::N_BITS == 0);

		U::BITS / Scalar::N_BITS
	};

	pub const LOG_WIDTH: usize = {
		let result = Self::WIDTH.ilog2();

		assert!(2usize.pow(result) == Self::WIDTH);

		result as usize
	};

	#[inline]
	pub fn from_underlier(val: U) -> Self {
		Self(val, PhantomData)
	}

	#[inline]
	pub const fn to_underlier(self) -> U {
		self.0
	}
}

unsafe impl<U: UnderlierType, Scalar: BinaryField> WithUnderlier
	for PackedPrimitiveType<U, Scalar>
{
	type Underlier = U;

	fn to_underlier(self) -> Self::Underlier {
		TransparentWrapper::peel(self)
	}

	fn to_underlier_ref(&self) -> &Self::Underlier {
		TransparentWrapper::peel_ref(self)
	}

	fn to_underlier_ref_mut(&mut self) -> &mut Self::Underlier {
		TransparentWrapper::peel_mut(self)
	}

	fn to_underliers_ref(val: &[Self]) -> &[Self::Underlier] {
		TransparentWrapper::peel_slice(val)
	}

	fn to_underliers_ref_mut(val: &mut [Self]) -> &mut [Self::Underlier] {
		TransparentWrapper::peel_slice_mut(val)
	}

	fn from_underlier(val: Self::Underlier) -> Self {
		TransparentWrapper::wrap(val)
	}

	fn from_underlier_ref(val: &Self::Underlier) -> &Self {
		TransparentWrapper::wrap_ref(val)
	}

	fn from_underlier_ref_mut(val: &mut Self::Underlier) -> &mut Self {
		TransparentWrapper::wrap_mut(val)
	}

	fn from_underliers_ref(val: &[Self::Underlier]) -> &[Self] {
		TransparentWrapper::wrap_slice(val)
	}

	fn from_underliers_ref_mut(val: &mut [Self::Underlier]) -> &mut [Self] {
		TransparentWrapper::wrap_slice_mut(val)
	}
}

impl<U: UnderlierWithBitOps, Scalar: BinaryField> Debug for PackedPrimitiveType<U, Scalar>
where
	Scalar::Underlier: NumCast<U>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let width = U::BITS / Scalar::N_BITS;
		let values: Vec<_> = (0..width)
			.map(|i| Scalar::from_underlier(self.0.get_subvalue(i)))
			.collect();
		write!(f, "Packed{}x{}({:?})", width, Scalar::N_BITS, values)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> From<U> for PackedPrimitiveType<U, Scalar> {
	#[inline]
	fn from(val: U) -> Self {
		Self(val, PhantomData)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> ConstantTimeEq for PackedPrimitiveType<U, Scalar> {
	fn ct_eq(&self, other: &Self) -> Choice {
		self.0.ct_eq(&other.0)
	}
}

impl<U: UnderlierWithBitOps, Scalar: BinaryField> Add for PackedPrimitiveType<U, Scalar> {
	type Output = Self;

	#[inline]
	#[allow(clippy::suspicious_arithmetic_impl)]
	fn add(self, rhs: Self) -> Self::Output {
		(self.0 ^ rhs.0).into()
	}
}

impl<U: UnderlierWithBitOps, Scalar: BinaryField> Sub for PackedPrimitiveType<U, Scalar> {
	type Output = Self;

	#[inline]
	#[allow(clippy::suspicious_arithmetic_impl)]
	fn sub(self, rhs: Self) -> Self::Output {
		(self.0 ^ rhs.0).into()
	}
}

impl<U: UnderlierType, Scalar: BinaryField> AddAssign for PackedPrimitiveType<U, Scalar>
where
	Self: Add<Output = Self>,
{
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl<U: UnderlierType, Scalar: BinaryField> SubAssign for PackedPrimitiveType<U, Scalar>
where
	Self: Sub<Output = Self>,
{
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl<U: UnderlierType, Scalar: BinaryField> MulAssign for PackedPrimitiveType<U, Scalar>
where
	Self: Mul<Output = Self>,
{
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}

impl<U: UnderlierType, Scalar: BinaryField> Add<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + Add<Output = Self>,
{
	type Output = Self;

	fn add(self, rhs: Scalar) -> Self::Output {
		self + Self::broadcast(rhs)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> Sub<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + Sub<Output = Self>,
{
	type Output = Self;

	fn sub(self, rhs: Scalar) -> Self::Output {
		self - Self::broadcast(rhs)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> Mul<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + Mul<Output = Self>,
{
	type Output = Self;

	fn mul(self, rhs: Scalar) -> Self::Output {
		self * Self::broadcast(rhs)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> AddAssign<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + AddAssign<Self>,
{
	fn add_assign(&mut self, rhs: Scalar) {
		*self += Self::broadcast(rhs);
	}
}

impl<U: UnderlierType, Scalar: BinaryField> SubAssign<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + SubAssign<Self>,
{
	fn sub_assign(&mut self, rhs: Scalar) {
		*self -= Self::broadcast(rhs);
	}
}

impl<U: UnderlierType, Scalar: BinaryField> MulAssign<Scalar> for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + MulAssign<Self>,
{
	fn mul_assign(&mut self, rhs: Scalar) {
		*self *= Self::broadcast(rhs);
	}
}

impl<U: UnderlierType, Scalar: BinaryField> Sum for PackedPrimitiveType<U, Scalar>
where
	Self: Add<Output = Self>,
{
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Self::from(U::default()), |result, next| result + next)
	}
}

impl<U: UnderlierType, Scalar: BinaryField> Product for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + Mul<Output = Self>,
{
	fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(Self::broadcast(Scalar::ONE), |result, next| result * next)
	}
}

unsafe impl<U: UnderlierType + Zeroable, Scalar: BinaryField> Zeroable
	for PackedPrimitiveType<U, Scalar>
{
}

unsafe impl<U: UnderlierType + Pod, Scalar: BinaryField> Pod for PackedPrimitiveType<U, Scalar> {}

impl<U: UnderlierWithBitOps, Scalar: BinaryField> PackedField for PackedPrimitiveType<U, Scalar>
where
	Self: Broadcast<Scalar> + Square + InvertOrZero + Mul<Output = Self>,
	U: UnderlierWithBitConstants + Send + Sync + 'static,
	Scalar: WithUnderlier<Underlier: UnderlierWithBitOps>,
	U: From<Scalar::Underlier>,
	Scalar::Underlier: NumCast<U>,
{
	type Scalar = Scalar;

	const LOG_WIDTH: usize = (U::BITS / Scalar::N_BITS).ilog2() as usize;

	#[inline]
	fn get_checked(&self, i: usize) -> Result<Self::Scalar, Error> {
		(i < Self::WIDTH)
			.then(|| self.0.get_subvalue(i))
			.ok_or(Error::IndexOutOfRange {
				index: i,
				max: Self::WIDTH,
			})
			.map(Scalar::from_underlier)
	}

	#[inline]
	fn set_checked(&mut self, i: usize, scalar: Scalar) -> Result<(), Error> {
		(i < Self::WIDTH)
			.then(|| self.0.set_subvalue(i, scalar.to_underlier()))
			.ok_or(Error::IndexOutOfRange {
				index: i,
				max: Self::WIDTH,
			})
	}

	fn random(rng: impl RngCore) -> Self {
		U::random(rng).into()
	}

	fn interleave(self, other: Self, log_block_len: usize) -> (Self, Self) {
		assert!(log_block_len < Self::LOG_WIDTH);
		let log_bit_len = Self::Scalar::N_BITS.ilog2() as usize;
		let (c, d) = self.0.interleave(other.0, log_block_len + log_bit_len);
		(c.into(), d.into())
	}

	#[inline]
	fn broadcast(scalar: Self::Scalar) -> Self {
		<Self as Broadcast<Self::Scalar>>::broadcast(scalar)
	}

	#[inline]
	fn from_fn(mut f: impl FnMut(usize) -> Self::Scalar) -> Self {
		U::from_fn(move |i| f(i).to_underlier()).into()
	}

	#[inline]
	fn square(self) -> Self {
		<Self as Square>::square(self)
	}

	#[inline]
	fn invert_or_zero(self) -> Self {
		<Self as InvertOrZero>::invert_or_zero(self)
	}
}

macro_rules! impl_broadcast {
	($name:ty, BinaryField1b) => {
		impl $crate::arithmetic_traits::Broadcast<BinaryField1b>
			for PackedPrimitiveType<$name, BinaryField1b>
		{
			#[inline]
			fn broadcast(scalar: BinaryField1b) -> Self {
				use $crate::underlier::{UnderlierWithBitOps, WithUnderlier};

				<Self as WithUnderlier>::Underlier::fill_with_bit(scalar.0.into()).into()
			}
		}
	};
	($name:ty, $scalar_type:ty) => {
		impl $crate::arithmetic_traits::Broadcast<$scalar_type>
			for PackedPrimitiveType<$name, $scalar_type>
		{
			#[inline]
			fn broadcast(scalar: $scalar_type) -> Self {
				let mut value = <$name>::from(scalar.0);
				// For PackedBinaryField1x128b, the log bits is 7, so this is
				// an empty range. This is safe behavior.
				#[allow(clippy::reversed_empty_ranges)]
				for i in <$scalar_type as $crate::binary_field::BinaryField>::N_BITS.ilog2()
					..<$name>::BITS.ilog2()
				{
					value = value << (1 << i) | value;
				}

				value.into()
			}
		}
	};
}

pub(crate) use impl_broadcast;

macro_rules! impl_ops_for_zero_height {
	($name:ty) => {
		impl std::ops::Mul for $name {
			type Output = Self;

			#[allow(clippy::suspicious_arithmetic_impl)]
			#[inline]
			fn mul(self, b: Self) -> Self {
				(self.to_underlier() & b.to_underlier()).into()
			}
		}

		impl $crate::arithmetic_traits::MulAlpha for $name {
			#[inline]
			fn mul_alpha(self) -> Self {
				self
			}
		}

		impl $crate::arithmetic_traits::Square for $name {
			#[inline]
			fn square(self) -> Self {
				self
			}
		}

		impl $crate::arithmetic_traits::InvertOrZero for $name {
			#[inline]
			fn invert_or_zero(self) -> Self {
				self
			}
		}
	};
}

pub(crate) use impl_ops_for_zero_height;

/// Multiply `PT1` values by upcasting to wider `PT2` type with the same scalar.
/// This is useful for the cases when SIMD multiplication is faster.
pub fn mul_as_bigger_type<PT1, PT2>(lhs: PT1, rhs: PT1) -> PT1
where
	PT1: PackedField + WithUnderlier,
	PT2: PackedField<Scalar = PT1::Scalar> + WithUnderlier,
	PT2::Underlier: From<PT1::Underlier>,
	PT1::Underlier: NumCast<PT2::Underlier>,
{
	let bigger_lhs = PT2::from_underlier(lhs.to_underlier().into());
	let bigger_rhs = PT2::from_underlier(rhs.to_underlier().into());

	let bigger_result = bigger_lhs * bigger_rhs;

	PT1::from_underlier(PT1::Underlier::num_cast_from(bigger_result.to_underlier()))
}

/// Square `PT1` values by upcasting to wider `PT2` type with the same scalar.
/// This is useful for the cases when SIMD square is faster.
pub fn square_as_bigger_type<PT1, PT2>(val: PT1) -> PT1
where
	PT1: PackedField + WithUnderlier,
	PT2: PackedField<Scalar = PT1::Scalar> + WithUnderlier,
	PT2::Underlier: From<PT1::Underlier>,
	PT1::Underlier: NumCast<PT2::Underlier>,
{
	let bigger_val = PT2::from_underlier(val.to_underlier().into());

	let bigger_result = bigger_val.square();

	PT1::from_underlier(PT1::Underlier::num_cast_from(bigger_result.to_underlier()))
}

/// Invert `PT1` values by upcasting to wider `PT2` type with the same scalar.
/// This is useful for the cases when SIMD invert is faster.
pub fn invert_as_bigger_type<PT1, PT2>(val: PT1) -> PT1
where
	PT1: PackedField + WithUnderlier,
	PT2: PackedField<Scalar = PT1::Scalar> + WithUnderlier,
	PT2::Underlier: From<PT1::Underlier>,
	PT1::Underlier: NumCast<PT2::Underlier>,
{
	let bigger_val = PT2::from_underlier(val.to_underlier().into());

	let bigger_result = bigger_val.invert_or_zero();

	PT1::from_underlier(PT1::Underlier::num_cast_from(bigger_result.to_underlier()))
}

/// Multiply by alpha `PT1` values by upcasting to wider `PT2` type with the same scalar.
/// This is useful for the cases when SIMD multiply by alpha is faster.
pub fn mul_alpha_as_bigger_type<PT1, PT2>(val: PT1) -> PT1
where
	PT1: PackedField + WithUnderlier,
	PT2: PackedField<Scalar = PT1::Scalar> + WithUnderlier + MulAlpha,
	PT2::Underlier: From<PT1::Underlier>,
	PT1::Underlier: NumCast<PT2::Underlier>,
{
	let bigger_val = PT2::from_underlier(val.to_underlier().into());

	let bigger_result = bigger_val.mul_alpha();

	PT1::from_underlier(PT1::Underlier::num_cast_from(bigger_result.to_underlier()))
}

macro_rules! impl_pack_scalar {
	($underlier:ty) => {
		impl<F> $crate::as_packed_field::PackScalar<F> for $underlier
		where
			F: BinaryField,
			PackedPrimitiveType<$underlier, F>:
				$crate::packed::PackedField<Scalar = F> + WithUnderlier<Underlier = $underlier>,
		{
			type Packed = PackedPrimitiveType<$underlier, F>;
		}
	};
}

pub(crate) use impl_pack_scalar;

impl_pack_scalar!(U1);
impl_pack_scalar!(U2);
impl_pack_scalar!(U4);
impl_pack_scalar!(u8);
impl_pack_scalar!(u16);
impl_pack_scalar!(u32);
impl_pack_scalar!(u64);
impl_pack_scalar!(u128);
