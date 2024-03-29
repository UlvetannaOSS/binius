// Copyright 2024 Ulvetanna Inc.

use crate::{
	field::TowerField,
	oracle::{MultilinearOracleSet, MultilinearPolyOracle},
};

use super::{
	error::Error,
	prodcheck::{reduce_prodcheck_claim, ProdcheckClaim, ReducedProductCheckClaims},
};

/// Verify a product check instance reduction.
pub fn verify<F: TowerField>(
	oracles: &mut MultilinearOracleSet<F>,
	claim: &ProdcheckClaim<F>,
	grand_prod_oracle: MultilinearPolyOracle<F>,
) -> Result<ReducedProductCheckClaims<F>, Error> {
	reduce_prodcheck_claim(oracles, claim, grand_prod_oracle)
}
