// Copyright 2023 Ulvetanna Inc.

use crate::{field::TowerField, oracle::MultilinearOracleSet, protocols::sumcheck::SumcheckClaim};

use super::{
	error::VerificationError,
	zerocheck::{reduce_zerocheck_claim, ZerocheckClaim, ZerocheckProof},
};

pub fn verify<F: TowerField>(
	oracles: &mut MultilinearOracleSet<F>,
	claim: &ZerocheckClaim<F>,
	proof: ZerocheckProof,
	challenge: Vec<F>,
) -> Result<SumcheckClaim<F>, VerificationError> {
	let _ = proof;
	reduce_zerocheck_claim(oracles, claim, challenge)
}
