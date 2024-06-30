use crate::{mock::*};

#[test]
fn it_works_for_basic_transfer() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		assert_eq!(System::block_number(), 1)
	});
}

