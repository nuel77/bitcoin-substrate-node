// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

//#[frame_support::pallet]
#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::{ensure_signed, pallet_prelude::*};
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    use sp_core::{
        sp_std::{collections::btree_map::BTreeMap, vec::Vec},
        H256,
    };
    use sp_runtime::traits::{BlakeTwo256, Hash};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(
        Clone,
        Encode,
        Decode,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        RuntimeDebug,
        Hash,
        Default,
        MaxEncodedLen,
        TypeInfo,
    )]
    pub struct Utxo<AccountId: Parameter + Member + MaxEncodedLen> {
        pub value: u64,
        /// owner of the utxo accountID
        pub owner: AccountId,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Hash, Default, TypeInfo)]
    pub struct TransactionInput {
        /// id of the utxo to be spend
        pub utxo_id: H256,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Hash, Default, TypeInfo)]
    pub struct Transaction<AccountId: Parameter + Member + MaxEncodedLen> {
        pub(crate) inputs: Vec<TransactionInput>,
        pub(crate) outputs: Vec<Utxo<AccountId>>,
    }

    impl<AccountId> Transaction<AccountId>
        where
            AccountId: Parameter + Member + MaxEncodedLen,
    {
        pub fn hash_input_utxo(&self, index: u64) -> H256 {
            BlakeTwo256::hash_of(&(&self.encode(), index))
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn utxo_store)]
    pub(super) type UtxoStore<T: Config> =
    StorageMap<_, Blake2_256, H256, Option<Utxo<T::AccountId>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully transferred a utxo.
        UtxoTransferred,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub supply: u64,
        pub owner: Option<T::AccountId>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(account) = &self.owner {
                let utxo = Utxo {
                    value: self.supply,
                    owner: account.clone()
                };
                UtxoStore::<T>::insert(H256::zero(), Some(utxo));
            }
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        DuplicatedTransaction,
        InvalidTransaction,
        OutputValueIsZero,
        InputsNotSatisfied,
        SignatureFailure,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            transaction: Transaction<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Check that the extrinsic was signed and get the signer.
            let validity = validate_tx::<T>(&transaction, who)?;
            ensure!(validity.requires.is_empty(), Error::<T>::InputsNotSatisfied);

            // Update storage.
            update_utxo_store::<T>(&transaction)?;

            // Emit an event.
            Self::deposit_event(Event::UtxoTransferred);

            // Return a successful `DispatchResult`
            Ok(())
        }
    }

    pub fn validate_tx<T: Config>(
        transaction: &Transaction<T::AccountId>,
        from: T::AccountId,
    ) -> Result<ValidTransaction, &'static str> {
        ensure!(transaction.inputs.len() > 0, "Transaction input length is 0");
        ensure!(transaction.outputs.len() > 0, "Transaction output length is 0");

        {
            // check if input transactions are not duplicated
            let mut map = BTreeMap::new();
            for input in &transaction.inputs {
                map.insert(input.utxo_id, ());
            }
            ensure!(map.len() == transaction.inputs.len(), "Duplicated Transaction Input");
        }

        {
            // check if output transactions are not duplicated
            let mut map = BTreeMap::new();
            for output in &transaction.outputs {
                map.insert(&output.owner, ());
            }
            ensure!(map.len() == transaction.outputs.len(), "Duplicated Transaction Output");
        }

        let mut missing_utxos = Vec::new();
        let mut created_utxos = Vec::new();

        for input in &transaction.inputs {
            //check if utxo exist in the store
            if let Some(utxo) = UtxoStore::<T>::get(input.utxo_id) {
                // check if all utxo are from the same account
                ensure!(from == utxo.owner, "Owner invalid");
            } else {
                missing_utxos.push(input.utxo_id.as_fixed_bytes().to_vec())
            }
        }

        // validate if output are valid (non 0)
        let mut idx: u64 = 0;
        for output in &transaction.outputs {
            ensure!(output.value > 0, "Output value is 0");
            let hash = transaction.hash_input_utxo(idx);
            idx = idx.saturating_add(1);
            created_utxos.push(hash.clone().as_fixed_bytes().to_vec())
        }

        Ok(ValidTransaction {
            priority: 1,
            requires: missing_utxos,
            provides: created_utxos,
            longevity: 10,
            propagate: true,
        })
    }

    pub fn update_utxo_store<T: Config>(transaction: &Transaction<T::AccountId>) -> DispatchResult {
        //remove outdated utxo
        for input in &transaction.inputs {
            UtxoStore::<T>::remove(input.utxo_id);
        }

        //add newly validated utxos
        let mut idx: u32 = 0;
        for output in &transaction.outputs {
            // create a unique and deterministic hash for each uxto in output
            // Do not use random here, as then the hash will be different for
            // other nodes in the network.
            let hash = transaction.hash_input_utxo(idx as u64);
            idx = idx.saturating_add(1);
            UtxoStore::<T>::insert(hash, Some(output.clone()));
        }

        Ok(())
    }
}
