// Copyright 2024 Ulvetanna Inc.

use super::{
	super::m256::M256,
	constants::{GFNI_TO_TOWER_MAP, TOWER_TO_GFNI_MAP},
	simd_arithmetic::TowerSimdType,
};
use crate::field::{
	arch::{
		portable::{
			packed::{
				impl_conversion, impl_ops_for_zero_height, impl_packed_extension_field,
				packed_binary_field_tower, PackedPrimitiveType,
			},
			packed_arithmetic::{alphas, impl_tower_constants},
		},
		PairwiseStrategy, SimdStrategy,
	},
	arithmetic_traits::{
		impl_invert_with_strategy, impl_mul_alpha_with_strategy, impl_mul_with_strategy,
		impl_square_with_strategy,
	},
	BinaryField128b, BinaryField16b, BinaryField1b, BinaryField2b, BinaryField32b, BinaryField4b,
	BinaryField64b, BinaryField8b, TowerField,
};
use std::{arch::x86_64::*, ops::Mul};

// Define 128 bit packed field types
pub type PackedBinaryField256x1b = PackedPrimitiveType<M256, BinaryField1b>;
pub type PackedBinaryField128x2b = PackedPrimitiveType<M256, BinaryField2b>;
pub type PackedBinaryField64x4b = PackedPrimitiveType<M256, BinaryField4b>;
pub type PackedBinaryField32x8b = PackedPrimitiveType<M256, BinaryField8b>;
pub type PackedBinaryField16x16b = PackedPrimitiveType<M256, BinaryField16b>;
pub type PackedBinaryField8x32b = PackedPrimitiveType<M256, BinaryField32b>;
pub type PackedBinaryField4x64b = PackedPrimitiveType<M256, BinaryField64b>;
pub type PackedBinaryField2x128b = PackedPrimitiveType<M256, BinaryField128b>;

// Define conversion from type to underlier
impl_conversion!(M256, PackedBinaryField256x1b);
impl_conversion!(M256, PackedBinaryField128x2b);
impl_conversion!(M256, PackedBinaryField64x4b);
impl_conversion!(M256, PackedBinaryField32x8b);
impl_conversion!(M256, PackedBinaryField16x16b);
impl_conversion!(M256, PackedBinaryField8x32b);
impl_conversion!(M256, PackedBinaryField4x64b);
impl_conversion!(M256, PackedBinaryField2x128b);

// Define tower
packed_binary_field_tower!(
	PackedBinaryField256x1b
	< PackedBinaryField128x2b
	< PackedBinaryField64x4b
	< PackedBinaryField32x8b
	< PackedBinaryField16x16b
	< PackedBinaryField8x32b
	< PackedBinaryField4x64b
	< PackedBinaryField2x128b
);

// Define extension fields
impl_packed_extension_field!(PackedBinaryField32x8b);
impl_packed_extension_field!(PackedBinaryField16x16b);
impl_packed_extension_field!(PackedBinaryField8x32b);
impl_packed_extension_field!(PackedBinaryField4x64b);
impl_packed_extension_field!(PackedBinaryField2x128b);

// Define operations for zero height
impl_ops_for_zero_height!(PackedBinaryField256x1b);

// Define constants
impl_tower_constants!(BinaryField1b, M256, { M256::from_equal_u128s(alphas!(u128, 0)) });
impl_tower_constants!(BinaryField2b, M256, { M256::from_equal_u128s(alphas!(u128, 1)) });
impl_tower_constants!(BinaryField4b, M256, { M256::from_equal_u128s(alphas!(u128, 2)) });
impl_tower_constants!(BinaryField8b, M256, { M256::from_equal_u128s(alphas!(u128, 3)) });
impl_tower_constants!(BinaryField16b, M256, { M256::from_equal_u128s(alphas!(u128, 4)) });
impl_tower_constants!(BinaryField32b, M256, { M256::from_equal_u128s(alphas!(u128, 5)) });
impl_tower_constants!(BinaryField64b, M256, { M256::from_equal_u128s(alphas!(u128, 6)) });

// Define multiplication
impl_mul_with_strategy!(PackedBinaryField128x2b, PairwiseStrategy);
impl_mul_with_strategy!(PackedBinaryField64x4b, PairwiseStrategy);
impl_mul_with_strategy!(PackedBinaryField16x16b, SimdStrategy);
impl_mul_with_strategy!(PackedBinaryField8x32b, SimdStrategy);
impl_mul_with_strategy!(PackedBinaryField4x64b, SimdStrategy);
impl_mul_with_strategy!(PackedBinaryField2x128b, SimdStrategy);

impl Mul for PackedBinaryField32x8b {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		unsafe {
			let tower_to_gfni_map = _mm256_set1_epi64x(TOWER_TO_GFNI_MAP);
			let gfni_to_tower_map = _mm256_set1_epi64x(GFNI_TO_TOWER_MAP);

			let lhs_gfni =
				_mm256_gf2p8affine_epi64_epi8::<0>(self.to_underlier().into(), tower_to_gfni_map);
			let rhs_gfni =
				_mm256_gf2p8affine_epi64_epi8::<0>(rhs.to_underlier().into(), tower_to_gfni_map);
			let prod_gfni = _mm256_gf2p8mul_epi8(lhs_gfni, rhs_gfni);
			M256::from(_mm256_gf2p8affine_epi64_epi8::<0>(prod_gfni, gfni_to_tower_map)).into()
		}
	}
}

// TODO: use more optimal SIMD implementation
// Define square
impl_square_with_strategy!(PackedBinaryField128x2b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField64x4b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField32x8b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField16x16b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField8x32b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField4x64b, PairwiseStrategy);
impl_square_with_strategy!(PackedBinaryField2x128b, PairwiseStrategy);

// TODO: use more optimal SIMD implementation
// Define invert
impl_invert_with_strategy!(PackedBinaryField128x2b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField64x4b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField32x8b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField16x16b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField8x32b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField4x64b, PairwiseStrategy);
impl_invert_with_strategy!(PackedBinaryField2x128b, PairwiseStrategy);

// TODO: use more optimal SIMD implementation
// Define multiply by alpha
impl_mul_alpha_with_strategy!(PackedBinaryField128x2b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField64x4b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField32x8b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField16x16b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField8x32b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField4x64b, PairwiseStrategy);
impl_mul_alpha_with_strategy!(PackedBinaryField2x128b, PairwiseStrategy);

impl TowerSimdType for M256 {
	fn xor(a: Self, b: Self) -> Self {
		unsafe { _mm256_xor_si256(a.0, b.0) }.into()
	}

	fn shuffle_epi8(a: Self, b: Self) -> Self {
		unsafe { _mm256_shuffle_epi8(a.0, b.0) }.into()
	}

	fn set_alpha_even<Scalar: TowerField>(self) -> Self {
		unsafe {
			let alpha = Self::alpha::<Scalar>();
			let mask = Self::even_mask::<Scalar>();
			// NOTE: There appears to be a bug in _mm_blendv_epi8 where the mask bit selects b, not a
			_mm256_blendv_epi8(self.0, alpha.0, mask.0)
		}
		.into()
	}

	fn set1_epi128(val: __m128i) -> Self {
		unsafe { _mm256_broadcastsi128_si256(val) }.into()
	}
}
