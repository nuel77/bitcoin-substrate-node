#![deny(unused_crate_dependencies)]

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[frame_support::pallet]
pub mod pallet {
    use frame_support::dispatch::DispatchResult;
    use frame_support::pallet_prelude::*;
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::*;
    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};
    use sp_runtime::traits::{BlakeTwo256, Hash};
    use sp_core::{
        sp_std::collections::btree_map::BTreeMap,
        H256
    };

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
        value: u64,
        /// owner of the utxo accountID
        owner: AccountId,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Hash, Default, TypeInfo)]
    pub struct TransactionInput {
        /// id of the utxo to be spend
        utxo_id: H256,
    }

    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Hash, Default, TypeInfo)]
    pub struct Transaction<AccountId: Parameter + Member + MaxEncodedLen> {
        pub(crate) inputs: Vec<TransactionInput>,
        pub(crate) outputs: Vec<Utxo<AccountId>>,
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

    #[pallet::error]
    pub enum Error<T> {
        DuplicatedTransaction,
        InvalidTransaction,
        OutputValueIsZero,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn transfer(origin: OriginFor<T>, transaction: Transaction<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Check that the extrinsic was signed and get the signer.
            Self::validate_tx(&transaction, &who)?;

            // Update storage.
            Self::update_utxo_store(&transaction)?;

            // Emit an event.
            Self::deposit_event(Event::UtxoTransferred);

            // Return a successful `DispatchResult`
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn validate_tx(transaction: &Transaction<T::AccountId>, who: &T::AccountId) -> Result<(), &'static str> {
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
                ensure!(map.len() == transaction.outputs.len(),"Duplicated Transaction Output");
            }

            // check if signature is valid and check if inputs are valid (exist in utxo store)
            for input in &transaction.inputs {
                if let Some(utxo) = UtxoStore::<T>::get(input.utxo_id) {
                    if utxo.owner != *who {
                        return Err("Owner of input and output does not match");
                    }
                } else {
                    return Err("Input does not exist in UTXO store");
                }
            }

            // validate if output are valid (non 0)
            for output in &transaction.outputs {
                ensure!(output.value > 0, "Output value is 0");
            }
            Ok(())
        }

        pub fn update_utxo_store(transaction: &Transaction<T::AccountId>) -> DispatchResult {
            //remove outdated utxo
            for input in &transaction.inputs {
                UtxoStore::<T>::remove(input.utxo_id);
            }

            //add newly validated utxos
            for output in &transaction.outputs {
                let hash = BlakeTwo256::hash_of(&output);
                UtxoStore::<T>::insert(hash, Some(output.clone()));
            }

            Ok(())
        }
    }
}
