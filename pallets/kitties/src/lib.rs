#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::dispatch::fmt;
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		price: u32,
		gender: Gender,
		account: T::AccountId,
	}

	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn student_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type NumberOfKitties<T> = StorageValue<_, u32, ValueQuery>;
	pub type NewOwner<T: Config> = T::AccountId;

	// key : dna
	//value : kitty
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner_to_kitties)]
	pub(super) type OwnerToKitties<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyStored(Vec<u8>, u32),
		TransferKittySuccess(Vec<u8>, NewOwner<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		TooShort,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		KittyNotFound,
		NotOwner
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	//extrinsic
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			// ensure!(dna.len() > 10, Error::<T>::TooShort);
			let gender = Self::gen_gender(dna.clone())?;
			let kitty = Kitty { dna: dna.clone(), price, gender, account: who.clone() };

			let mut kitties = <OwnerToKitties<T>>::get(who.clone()).unwrap_or(Vec::new());
			kitties.push(dna.clone());

			<OwnerToKitties<T>>::insert(who.clone(), kitties);

			let mut number_of_kitties = <NumberOfKitties<T>>::get();
			<Kitties<T>>::insert(dna.clone(), kitty);
			number_of_kitties += 1;
			NumberOfKitties::<T>::put(number_of_kitties);
			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, price));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_ownership(origin: OriginFor<T>, dna: Vec<u8>, newOwner: NewOwner<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty = <Kitties<T>>::get(dna.clone());
			ensure!(kitty.is_some(), Error::<T>::KittyNotFound);
			let mut kitty = kitty.unwrap();
			ensure!(kitty.account == who, Error::<T>::NotOwner);


			// Assign new owner for kitty
			kitty.account = newOwner.clone();
			<Kitties<T>>::insert(dna.clone(), kitty);

			// Update number of kitties owned by new owner
			let mut kitties = <OwnerToKitties<T>>::get(newOwner.clone()).unwrap_or(Vec::new());
			kitties.push(dna.clone());

			<OwnerToKitties<T>>::insert(newOwner.clone(), kitties);

			// Remove kitty from old owner
			let mut kitties = <OwnerToKitties<T>>::get(who.clone()).unwrap_or(Vec::new());
			kitties.retain(|x| x != &dna);
			<OwnerToKitties<T>>::insert(who.clone(), kitties);
			// Emit an event.
			Self::deposit_event(Event::TransferKittySuccess(dna, newOwner));

			Ok(())
		}


		
	}
}

// helper function

impl<T> Pallet<T> {
	fn gen_gender(name: Vec<u8>) -> Result<Gender, Error<T>> {
		let mut res = Gender::Male;
		if name.len() % 2 == 0 {
			res = Gender::Female;
		}
		Ok(res)
	}
}

// Tóm tắt:
//Custom type: Struct ,Enum
// Sử dụng generic type đối với trait
// helper function
// origin
// một số method cơ bản liên quan tới read/write storage
// giải quuêys một số bug có thể có .
