#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::DispatchResult;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = r#"runtime-benchmarks"#)]
// mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_timestamp::{self as timestamp};
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;

	#[derive(TypeInfo, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Campaign<T: Config> {
		pub id: u32,
		pub title: Vec<u8>,
		pub description: Vec<u8>,
		pub goal: u32,
		pub balance: u32,
		pub beneficiary: T::AccountId,
		pub start_date: <T as pallet_timestamp::Config>::Moment,
		pub end_date: <T as pallet_timestamp::Config>::Moment,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	#[pallet::storage]
	#[pallet::getter(fn nb_campaigns)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type NumberOfCampaign<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub(super) type Campaigns<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, Campaign<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		CampaignStored(u32, T::AccountId),
		DonateSuccess(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		CampaignNotFound,
		AmountOverflowGoal,
		/// Errors should have helpful documentation associated with them.
		OutOfPeriod,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Create campaign
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_campaign(
			origin: OriginFor<T>,
			title: Vec<u8>,
			description: Vec<u8>,
			goal: u32,
			beneficiary: T::AccountId,
			start_date: <T as pallet_timestamp::Config>::Moment,
			end_date: <T as pallet_timestamp::Config>::Moment,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut id = <NumberOfCampaign<T>>::get();
			id += 1;
			<NumberOfCampaign<T>>::put(id);
			let campaign = Campaign {
				id,
				title,
				description,
				goal,
				balance: 0u32,
				beneficiary,
				start_date,
				end_date,
			};
			<Campaigns<T>>::insert(id, campaign);
			Self::deposit_event(Event::CampaignStored(id, sender));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn donate_to_campaign(
			origin: OriginFor<T>,
			campaign_id: u32,
			amount: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let campaign = <Campaigns<T>>::get(campaign_id);
			ensure!(campaign.is_some(), Error::<T>::CampaignNotFound);
			let mut campaign = campaign.unwrap();
			ensure!(campaign.start_date <= <timestamp::Pallet<T>>::get(), Error::<T>::OutOfPeriod);
			ensure!(campaign.end_date >= <timestamp::Pallet<T>>::get(), Error::<T>::OutOfPeriod);
			ensure!(campaign.goal - campaign.balance >= amount, Error::<T>::AmountOverflowGoal);
			campaign.balance += amount;
			<Campaigns<T>>::insert(campaign_id, campaign);
			Self::deposit_event(Event::DonateSuccess(amount, who));
			Ok(())
		}
	}
}
