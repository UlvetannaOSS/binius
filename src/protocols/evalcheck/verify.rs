// Copyright 2024 Ulvetanna Inc.

use crate::{
	field::{util::inner_product_unchecked, Field},
	oracle::{MultilinearPolyOracle, MultivariatePolyOracle, ProjectionVariant},
	polynomial::extrapolate_line,
	protocols::evalcheck::evalcheck::ShiftedEvalClaim,
};
use tracing::instrument;

use super::{
	error::{Error, VerificationError},
	evalcheck::{
		BatchCommittedEvalClaims, CommittedEvalClaim, EvalcheckClaim, EvalcheckProof,
		PackedEvalClaim,
	},
};

#[instrument(skip_all, name = "evalcheck::verify")]
pub fn verify<F: Field>(
	evalcheck_claim: EvalcheckClaim<F>,
	evalcheck_proof: EvalcheckProof<F>,
	batch_commited_eval_claims: &mut BatchCommittedEvalClaims<F>,
	shifted_eval_claims: &mut Vec<ShiftedEvalClaim<F>>,
	packed_eval_claims: &mut Vec<PackedEvalClaim<F>>,
) -> Result<(), Error> {
	verify_inner(
		evalcheck_claim,
		evalcheck_proof,
		batch_commited_eval_claims,
		shifted_eval_claims,
		packed_eval_claims,
	)
}

pub fn verify_inner<F: Field>(
	evalcheck_claim: EvalcheckClaim<F>,
	evalcheck_proof: EvalcheckProof<F>,
	batch_commited_eval_claims: &mut BatchCommittedEvalClaims<F>,
	shifted_eval_claims: &mut Vec<ShiftedEvalClaim<F>>,
	packed_eval_claims: &mut Vec<PackedEvalClaim<F>>,
) -> Result<(), Error> {
	let EvalcheckClaim {
		poly,
		mut eval_point,
		eval,
		is_random_point,
	} = evalcheck_claim;

	match poly {
		MultivariatePolyOracle::Multilinear(multilinear) => match multilinear {
			MultilinearPolyOracle::Transparent(_id, transparent) => {
				match evalcheck_proof {
					EvalcheckProof::Transparent => {}
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				let actual_eval = transparent.poly().evaluate(&eval_point)?;
				if actual_eval != eval {
					return Err(VerificationError::IncorrectEvaluation.into());
				}
			}
			MultilinearPolyOracle::Committed { id, .. } => {
				match evalcheck_proof {
					EvalcheckProof::Committed => {}
					_ => return Err(VerificationError::SubproofMismatch.into()),
				}

				let subclaim = CommittedEvalClaim {
					id,
					eval_point,
					eval,
					is_random_point,
				};

				batch_commited_eval_claims.insert(subclaim)?;
			}
			MultilinearPolyOracle::Repeating { inner, .. } => {
				let subproof = match evalcheck_proof {
					EvalcheckProof::Repeating(subproof) => subproof,
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				let n_vars = inner.n_vars();
				let subclaim = EvalcheckClaim {
					poly: MultivariatePolyOracle::Multilinear(*inner),
					eval_point: eval_point[..n_vars].to_vec(),
					eval,
					is_random_point,
				};

				verify_inner(
					subclaim,
					*subproof,
					batch_commited_eval_claims,
					shifted_eval_claims,
					packed_eval_claims,
				)?;
			}
			MultilinearPolyOracle::Interleaved(..) => {
				// TODO: Implement interleaved reduction, similar to merged
				todo!()
			}
			MultilinearPolyOracle::Merged(_id, poly1, poly2) => {
				let (eval1, eval2, subproof1, subproof2) = match evalcheck_proof {
					EvalcheckProof::Merged {
						eval1,
						eval2,
						subproof1,
						subproof2,
					} => (eval1, eval2, subproof1, subproof2),
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				// Verify the evaluation of the merged function over the claimed evaluations
				let n_vars = poly1.n_vars();
				let subclaim_eval_point = &eval_point[..n_vars];
				let actual_eval = extrapolate_line(eval1, eval2, eval_point[n_vars]);
				if actual_eval != eval {
					return Err(VerificationError::IncorrectEvaluation.into());
				}

				let claim1 = EvalcheckClaim {
					poly: MultivariatePolyOracle::Multilinear(*poly1),
					eval_point: subclaim_eval_point.to_vec(),
					eval: eval1,
					is_random_point,
				};
				verify_inner(
					claim1,
					*subproof1,
					batch_commited_eval_claims,
					shifted_eval_claims,
					packed_eval_claims,
				)?;

				let claim2 = EvalcheckClaim {
					poly: MultivariatePolyOracle::Multilinear(*poly2),
					eval_point: subclaim_eval_point.to_vec(),
					eval: eval2,
					is_random_point,
				};
				verify_inner(
					claim2,
					*subproof2,
					batch_commited_eval_claims,
					shifted_eval_claims,
					packed_eval_claims,
				)?;
			}
			MultilinearPolyOracle::Projected(_id, projected) => {
				let (inner, values) = (projected.inner(), projected.values());
				let eval_point = match projected.projection_variant() {
					ProjectionVariant::LastVars => {
						eval_point.extend(values);
						eval_point
					}
					ProjectionVariant::FirstVars => {
						values.iter().cloned().chain(eval_point).collect()
					}
				};
				let new_claim = EvalcheckClaim {
					poly: MultivariatePolyOracle::Multilinear(*inner.clone()),
					eval_point,
					eval,
					is_random_point,
				};
				verify_inner(
					new_claim,
					evalcheck_proof,
					batch_commited_eval_claims,
					shifted_eval_claims,
					packed_eval_claims,
				)?;
			}
			MultilinearPolyOracle::Shifted(_id, shifted) => {
				match evalcheck_proof {
					EvalcheckProof::Shifted => {}
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				let subclaim = ShiftedEvalClaim {
					eval_point,
					eval,
					is_random_point,
					shifted,
				};
				shifted_eval_claims.push(subclaim);
			}
			MultilinearPolyOracle::Packed(_id, packed) => {
				match evalcheck_proof {
					EvalcheckProof::Packed => {}
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				let subclaim = PackedEvalClaim {
					eval_point,
					eval,
					is_random_point,
					packed,
				};
				packed_eval_claims.push(subclaim);
			}
			MultilinearPolyOracle::LinearCombination(_id, lin_com) => {
				let subproofs = match evalcheck_proof {
					EvalcheckProof::Composite { subproofs } => subproofs,
					_ => return Err(VerificationError::SubproofMismatch.into()),
				};

				if subproofs.len() != lin_com.n_polys() {
					return Err(VerificationError::SubproofMismatch.into());
				}

				// Verify the evaluation of the linear combination over the claimed evaluations
				let actual_eval = inner_product_unchecked(
					subproofs.iter().map(|(eval, _)| *eval),
					lin_com.coefficients(),
				);
				if actual_eval != eval {
					return Err(VerificationError::IncorrectEvaluation.into());
				}

				subproofs.into_iter().zip(lin_com.polys()).try_for_each(
					|((eval, subproof), suboracle)| {
						let subclaim = EvalcheckClaim {
							poly: MultivariatePolyOracle::Multilinear(suboracle.clone()),
							eval_point: eval_point.clone(),
							eval,
							is_random_point,
						};
						verify_inner(
							subclaim,
							subproof,
							batch_commited_eval_claims,
							shifted_eval_claims,
							packed_eval_claims,
						)
					},
				)?;
			}
		},
		MultivariatePolyOracle::Composite(composite) => {
			let subproofs = match evalcheck_proof {
				EvalcheckProof::Composite { subproofs } => subproofs,
				_ => return Err(VerificationError::SubproofMismatch.into()),
			};

			if subproofs.len() != composite.n_multilinears() {
				return Err(VerificationError::SubproofMismatch.into());
			}

			// Verify the evaluation of the composition function over the claimed evaluations
			let evals = subproofs.iter().map(|(eval, _)| *eval).collect::<Vec<_>>();
			let actual_eval = composite.composition().evaluate(&evals)?;
			if actual_eval != eval {
				return Err(VerificationError::IncorrectEvaluation.into());
			}

			subproofs
				.into_iter()
				.zip(composite.inner_polys().into_iter())
				.try_for_each(|((eval, subproof), suboracle)| {
					let subclaim = EvalcheckClaim {
						poly: MultivariatePolyOracle::Multilinear(suboracle),
						eval_point: eval_point.clone(),
						eval,
						is_random_point,
					};
					verify_inner(
						subclaim,
						subproof,
						batch_commited_eval_claims,
						shifted_eval_claims,
						packed_eval_claims,
					)
				})?;
		}
	}

	Ok(())
}
