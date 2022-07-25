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

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
// use frame_support::inherent::Vec;
// use frame_support::dispatch::fmt;
use sp_std::vec::Vec;
use scale_info::TypeInfo;
pub type Id = u32;
use frame_support::traits::{Currency, Time};
use sp_runtime::ArithmeticError;
use frame_support::traits::*;
use pallet_timestamp;
use sp_runtime::traits::*;
use frame_support::dispatch::fmt::Debug;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	

// use pallet_timestamp as timestamp;
	pub use super::*;
	

	#[derive(TypeInfo, PartialEq, Encode, Decode, Clone, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T:Config> {
		pub dna: T::Hash,
		pub price: BalanceOf<T>,
		pub gender: Gender,
		pub owner: T::AccountId,
		pub create_date: <<T as Config>::CreateKitty as Time>::Moment,
	}
	

	#[derive(TypeInfo, Encode ,Decode, Clone, Copy, RuntimeDebug, MaxEncodedLen, PartialEq)]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type CreateKitty: Time;
		type MaxOwned: Get<u32>;
		type RandomKitty: Randomness<Self::Hash, Self::BlockNumber>;
	}
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyId<T> = StorageValue<_, Id,ValueQuery>;

	// #[pallet::storage]
	// #[pallet::getter(fn kitty_time)]
	// pub type KittyTime<T> = StorageValue<_, u64, ValueQuery>;

	// key : id
	//value : student
	#[pallet::storage]
	#[pallet::getter(fn get_kitty)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat,T::AccountId , BoundedVec<T::Hash, T::MaxOwned>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_total)]
	pub(super) type KittiesTotal<T: Config> = StorageMap<_, Blake2_128Concat,T::AccountId , Vec<BalanceOf<T>>, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T:Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Created { kitty: T::Hash, owner: T::AccountId },
		Transferred { from: T::AccountId, to: T::AccountId, kitty:T::Hash },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		DuplicateKitty,
		TooManyOwned,
		NoKitty,
		NotOwner,
		TransferToSelf,
		TooCheap,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	//extrinsic
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: BalanceOf<T>) -> DispatchResult {
			
			let who = ensure_signed(origin)?;
			let now = T::CreateKitty::now();
			//let current_time = T::Time::now().as_millis().saturated_into::<u64>();
			let gender = Self::gen_gender(&dna)?;
			let dna = Self::gen_dna();
			// log::info!("{:?}", time);
			
			
			let kitty = Kitty::<T>{dna: dna.clone(), price: price, gender, owner: who.clone(), create_date: now};
			let maxkitty = T::MaxOwned::get();
			let get_kitties = KittiesOwned::<T>::get(&who);
			ensure!((get_kitties.len() as u32) < maxkitty, Error::<T>::TooManyOwned);

			//check if the kitty does not already exits in our storage map
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);
			let current_id = <KittyId<T>>::get();
			let next_id = current_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			KittiesOwned::<T>::try_append(&who, kitty.dna.clone()).map_err(|_| Error::<T>::TooManyOwned)?;
		
			Kitties::<T>::insert(kitty.dna.clone(), kitty);
			KittyId::<T>::put(next_id);
			Self::deposit_event(Event::Created{kitty: dna, owner: who.clone()});
		
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_owner(origin: OriginFor<T>,kitty_id: T::Hash, new_owner: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			//let create_date = <timestamp::Pallet<T>>::get() as u64;
			let kitty = <Kitties<T>>::get(kitty_id.clone()).unwrap();
			
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);
			let new_kitty = Kitty {
				dna: kitty.dna,
				price: kitty.price,
				gender: kitty.gender,
				owner: new_owner,
				create_date: kitty.create_date
			};
			<Kitties<T>>::insert(kitty_id.clone(), new_kitty);
			Ok(())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_price(origin: OriginFor<T>,kitty_id: T::Hash, new_price: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let kitty = <Kitties<T>>::get(kitty_id.clone()).unwrap();
			ensure!(who == kitty.owner, Error::<T>::NoKitty);
			let new_kitty = Kitty{
				dna: kitty.dna,
				price: new_price,
				gender: kitty.gender,
				owner: kitty.owner,
				create_date: kitty.create_date,
			};
			//<Kitties<T>>::remove(kitty_id.clone());
			<Kitties<T>>::insert(kitty_id.clone(), new_kitty);
			
			Ok(())
		}
		
		#[pallet::weight(0)]
		pub fn transfer_kitty(origin: OriginFor<T>,reciever: T::AccountId,dna: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut kitty = <Kitties<T>>::get(&dna).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);
			ensure!(sender != reciever, Error::<T>::TransferToSelf);
			//let create_date = <timestamp::Pallet<T>>::get() as u64;
			let mut receiver_owned = KittiesOwned::<T>::get(&reciever);
			//remove kitty from list of owned kitties
			if let Some(ind) = receiver_owned.iter().position(|x| *x == dna) {
				receiver_owned.remove(ind);
			}
			else {
				return Err(Error::<T>::NoKitty.into());
			}

			let mut sender_owned = KittiesOwned::<T>::get(&sender);
			sender_owned.try_push(dna.clone()).map_err(|_| Error::<T>::TooManyOwned)?;
			kitty.owner = sender.clone();

			//write update to storage
			Kitties::<T>::insert(dna.clone(), kitty);

			KittiesOwned::<T>::insert(&sender.clone(), sender_owned);
			KittiesOwned::<T>::insert(&reciever.clone(), receiver_owned);

			Self::deposit_event(Event::Transferred{from: sender, to: reciever, kitty: dna});
			Ok(())
		}
		
	}
}

impl<T: Config> Pallet<T> {
	
	fn gen_gender(dna: &Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Female;
		if dna.len() % 2 ==0 {
			res = Gender::Male;
		}
		
		Ok(res)
	}
	fn gen_dna() -> T::Hash {
		let (seed, _) = T::RandomKitty::random_seed();
		let block_number = <frame_system::Pallet<T>>::block_number();
		T::Hashing::hash_of(&(seed, block_number))
	}
}
