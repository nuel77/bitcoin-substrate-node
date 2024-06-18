#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::DispatchResult;
	use frame_support::pallet_prelude::*;
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::*;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_core::crypto::AccountId32;
	use sp_core::{H256, H512};

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
	pub struct Utxo {
		value: u64,
		owner: H256,
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Hash, Default, TypeInfo)]
	pub struct Transaction {
		/// id of the utxo to be spend
		utxo_id: H256,
		signature: H512,
	}

	#[pallet::storage]
	#[pallet::getter(fn utxo_store)]
	pub(super) type UtxoStore<T: Config> =
		StorageMap<_, Blake2_256, H256, Option<Utxo>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully transferred a utxo.
		UtxoTransferred { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn transfer(origin: OriginFor<T>, transaction: Vec<Transaction>) -> DispatchResult {
			//ensure_signed(&origin)?;
			// Check that the extrinsic was signed and get the signer.
			let is_valid = Self::validate_tx(&transaction)?;

			// Update storage.
			Self::update_utxo_store(&transaction)?;

			// Emit an event.
			// Self::deposit_event(Event::UtxoTransferred { who: origin.into() });

			// Return a successful `DispatchResult`
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn validate_tx(transaction: &Vec<Transaction>) -> DispatchResult {
			Ok(())
		}

		pub fn update_utxo_store(transaction: &Vec<Transaction>) -> DispatchResult {
			Ok(())
		}
	}
}
