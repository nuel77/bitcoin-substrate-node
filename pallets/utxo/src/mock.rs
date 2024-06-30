use crate as pallet_utxo;
use frame_support::{
    derive_impl,
    traits::{ConstU16, ConstU64},
};
use frame_system::Config;
use sp_core::crypto::{AccountId32, KeyTypeId};
use sp_core::H256;
use sp_keystore::{Keystore, KeystoreExt};
use sp_keystore::testing::MemoryKeystore;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Utxo: pallet_utxo
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_utxo::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    pub const UTXO_KEY: KeyTypeId = KeyTypeId(*b"utxo");

    // keystore externality for test
    let keystore = MemoryKeystore::new();
    let alice_pub = keystore.sr25519_generate_new(UTXO_KEY, Some("//Alice")).unwrap();

    //create genesis config
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_utxo::GenesisConfig::<Test> {
        supply: 21000u64,
        owner: Some(alice_pub.into()),
    }
        .assimilate_storage(&mut t).unwrap();

    let mut ext = sp_io::TestExternalities::from(t);
    ext.register_extension(KeystoreExt(std::sync::Arc::new(keystore)));
    ext
}
