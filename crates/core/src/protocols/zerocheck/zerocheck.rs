// Copyright 2023 Ulvetanna Inc.

use super::error::VerificationError;
use crate::{
	oracle::{CompositePolyOracle, MultilinearOracleSet, OracleId},
	polynomial::{
		transparent::eq_ind::EqIndPartialEval, CompositionPoly, Error as PolynomialError,
		MultilinearComposite,
	},
	protocols::sumcheck::{SumcheckClaim, SumcheckWitness},
	witness::MultilinearWitness,
};
use binius_field::{Field, PackedField, TowerField};
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct ZerocheckProof;

#[derive(Debug)]

pub struct ZerocheckProveOutput<'a, F: Field, PW: PackedField, C, CW> {
	pub sumcheck_claim: SumcheckClaim<F, C>,
	pub sumcheck_witness: SumcheckWitness<PW, CW, MultilinearWitness<'a, PW>>,
	pub zerocheck_proof: ZerocheckProof,
}

#[derive(Debug, Clone)]
pub struct ZerocheckClaim<F: Field, C> {
	/// Virtual Polynomial Oracle of the function claimed to be zero on hypercube
	pub poly: CompositePolyOracle<F, C>,
}

/// Polynomial must be representable as a composition of multilinear polynomials
pub type ZerocheckWitness<'a, P, C> = MultilinearComposite<P, C, MultilinearWitness<'a, P>>;

/// This wraps an inner composition polynomial $f$ and multiplies by another variable..
///
/// The function is $g(X_0, ..., X_n) = f(X_0, ..., X_{n-1}) * X_n$.
#[derive(Clone, Debug)]
pub struct ProductComposition<C> {
	inner: C,
}

impl<C> ProductComposition<C> {
	pub fn new(inner: C) -> Self {
		Self { inner }
	}
}

impl<P: PackedField, C: CompositionPoly<P>> CompositionPoly<P> for ProductComposition<C> {
	fn n_vars(&self) -> usize {
		self.inner.n_vars() + 1
	}

	fn degree(&self) -> usize {
		self.inner.degree() + 1
	}

	fn evaluate(&self, query: &[P::Scalar]) -> Result<P::Scalar, PolynomialError> {
		let n_vars = self.n_vars();
		if query.len() != n_vars {
			return Err(PolynomialError::IncorrectQuerySize { expected: n_vars });
		}

		let inner_query = &query[..n_vars - 1];
		let inner_eval = self.inner.evaluate(inner_query)?;
		Ok(inner_eval * query[n_vars - 1])
	}

	fn evaluate_packed(&self, query: &[P]) -> Result<P, PolynomialError> {
		let n_vars = self.n_vars();
		if query.len() != n_vars {
			return Err(PolynomialError::IncorrectQuerySize { expected: n_vars });
		}

		let inner_query = &query[..n_vars - 1];
		let inner_eval = self.inner.evaluate_packed(inner_query)?;
		Ok(inner_eval * query[n_vars - 1])
	}

	fn binary_tower_level(&self) -> usize {
		self.inner.binary_tower_level()
	}
}

pub fn reduce_zerocheck_claim<F: TowerField, C: CompositionPoly<F>>(
	oracles: &mut MultilinearOracleSet<F>,
	claim: &ZerocheckClaim<F, C>,
	challenge: Vec<F>,
) -> Result<(SumcheckClaim<F, ProductComposition<C>>, OracleId), VerificationError> {
	if claim.poly.n_vars() != challenge.len() {
		return Err(VerificationError::ChallengeVectorMismatch);
	}

	let eq_r_multilinear = EqIndPartialEval::new(claim.poly.n_vars(), challenge)?;
	let eq_r_oracle_id = oracles.add_transparent(Arc::new(eq_r_multilinear), F::TOWER_LEVEL)?;

	let poly_composite = &claim.poly;
	let mut inners = poly_composite.inner_polys();
	inners.push(oracles.oracle(eq_r_oracle_id));

	let new_composition = ProductComposition::new(poly_composite.composition());
	let composite_poly = CompositePolyOracle::new(claim.poly.n_vars(), inners, new_composition)?;

	let sumcheck_claim = SumcheckClaim {
		poly: composite_poly,
		sum: F::ZERO,
	};
	Ok((sumcheck_claim, eq_r_oracle_id))
}