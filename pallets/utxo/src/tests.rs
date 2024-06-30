use crate as pallet_utxo;
use frame_support::assert_ok;
use sp_core::H256;
use crate::{mock::*, TransactionInput};

#[test]
fn it_works_for_basic_transfer() {
	new_test_ext().execute_with(|| {
		let alice = sp_io::crypto::sr25519_public_keys(UTXO_KEY)[0];
		let bob = sp_io::crypto::sr25519_public_keys(UTXO_KEY)[1];
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		assert_eq!(System::block_number(), 1);
		let genesis_utxo_id = H256::zero();
		// create a transfer transaction
		let tx = pallet_utxo::Transaction{
			inputs: vec![TransactionInput::new(genesis_utxo_id)],
			outputs: vec![pallet_utxo::Utxo::new(250, bob.into())],
		};
		assert_ok!(Utxo::transfer(RuntimeOrigin::signed(alice.into()), tx));
	});
}

