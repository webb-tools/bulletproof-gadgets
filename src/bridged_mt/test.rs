use crate::{
	poseidon::{
		sbox::PoseidonSbox, PoseidonBuilder, Poseidon_hash_2, Poseidon_hash_4,
	},
	smt::builder::SparseMerkleTreeBuilder,
};

use curve25519_dalek::scalar::Scalar;

use rand_core::OsRng;

#[test]
fn test_time_based_reward_gadget_verification() {
	let width = 6;
	let p_params = PoseidonBuilder::new(width)
		.sbox(PoseidonSbox::Inverse)
		.build();

	let mut test_rng = OsRng::default();
	let destination_chain = Scalar::from(2u32);

	// MAKE THE ORIGIN CHAIN TREE
	// - do a deposit at the leaf at index 7 destined for the DESTINATION CHAIN
	let origin_r = Scalar::random(&mut test_rng);
	let origin_nullifier = Scalar::random(&mut test_rng);
	// we want to do a bridge transfer so we commit to a leaf with the
	// destination_chain ID
	let origin_expected_output = Poseidon_hash_4(
		[destination_chain, origin_r, origin_r, origin_nullifier],
		&p_params,
	);
	let _origin_nullifier_hash =
		Poseidon_hash_2(origin_nullifier, origin_nullifier, &p_params);

	let mut origin_deposit_tree = SparseMerkleTreeBuilder::new()
		.hash_params(p_params.clone())
		.build();

	for i in 1..=10 {
		let index = Scalar::from(i as u32);
		let s = if i == 7 {
			origin_expected_output
		} else {
			index
		};

		origin_deposit_tree.update(index, s);
	}

	let mut origin_merkle_proof_vec = Vec::<Scalar>::new();
	let mut origin_merkle_proof = Some(origin_merkle_proof_vec);
	let k = Scalar::from(7u32);
	assert_eq!(
		origin_expected_output,
		origin_deposit_tree.get(
			k,
			origin_deposit_tree.root,
			&mut origin_merkle_proof
		)
	);
	origin_merkle_proof_vec = origin_merkle_proof.unwrap();
	assert!(origin_deposit_tree.verify_proof(
		k,
		origin_expected_output,
		&origin_merkle_proof_vec,
		None
	));
	assert!(origin_deposit_tree.verify_proof(
		k,
		origin_expected_output,
		&origin_merkle_proof_vec,
		Some(&origin_deposit_tree.root)
	));

	// MAKE THE DESTINATION CHAIN TREE
	let mut destination_deposit_tree = SparseMerkleTreeBuilder::new()
		.hash_params(p_params.clone())
		.build();

	for i in 1..=10 {
		let index = Scalar::from(i as u32);
		destination_deposit_tree.update(index, Scalar::random(&mut test_rng));
	}
}
