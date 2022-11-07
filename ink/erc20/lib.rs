#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Decode, Encode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	BalanceNotEnough,
	AllowanceNotEnough,
	FrozenFundNotEnough,
}

#[ink::contract]
mod erc20 {

	use crate::Error;
	use ink::prelude::string::String;
	use ink::storage::Mapping;

	pub type Result<T> = core::result::Result<T, Error>;

	#[ink::trait_definition]
	pub trait BaseErc20 {
		#[ink(message)]
		fn name(&self) -> String;

		#[ink(message)]
		fn symbol(&self) -> String;

		#[ink(message)]
		fn decimals(&self) -> u8;

		#[ink(message)]
		fn total_supply(&self) -> Balance;

		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> Balance;

		#[ink(message)]
		fn transfer(&mut self, to: AccountId, value: Balance) -> Result<bool>;

		#[ink(message)]
		fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance)
			-> Result<bool>;

		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: Balance) -> bool;

		#[ink(message)]
		fn allowance(&mut self, owner: AccountId, spender: AccountId) -> Balance;

		#[ink(message)]
		fn burn(&mut self, value: Balance) -> Result<bool>;

		#[ink(message)]
		fn freeze(&mut self, value: Balance) -> Result<bool>;

		#[ink(message)]
		fn unfreeze(&mut self, value: Balance) -> Result<bool>;
	}

	#[ink(storage)]
	#[derive(Default)]
	pub struct Erc20 {
		total_supply: Balance,
		balances: Mapping<AccountId, Balance>,
		allowances: Mapping<(AccountId, AccountId), Balance>,
		freezes: Mapping<AccountId, Balance>,
	}

	#[ink(event)]
	pub struct Transfer {
		#[ink(topic)]
		from: Option<AccountId>,
		#[ink(topic)]
		to: Option<AccountId>,
		value: Balance,
	}

	#[ink(event)]
	pub struct Approval {
		#[ink(topic)]
		owner: AccountId,
		#[ink(topic)]
		spender: AccountId,
		value: Balance,
	}

	#[ink(event)]
	pub struct Burn {
		#[ink(topic)]
		owner: AccountId,
		value: Balance,
		total_supply: Balance,
	}

	#[ink(event)]
	pub struct Freeze {
		#[ink(topic)]
		owner: AccountId,
		value: Balance,
	}

	#[ink(event)]
	pub struct Unfreeze {
		#[ink(topic)]
		owner: AccountId,
		value: Balance,
	}

	impl Erc20 {
		#[ink(constructor)]
		pub fn new(total_supply: Balance) -> Self {
			let caller = Self::env().caller();
			let mut balances = Mapping::default();
			balances.insert(&caller, &total_supply);

			Self::env().emit_event(Transfer { from: None, to: Some(caller), value: total_supply });
			Self { total_supply, balances, ..Default::default() }
		}

		#[ink(constructor)]
		pub fn default() -> Self {
			Default::default()
		}
	}

	impl BaseErc20 for Erc20 {
		#[ink(message)]
		fn name(&self) -> String {
			String::from("erc20")
		}

		#[ink(message)]
		fn symbol(&self) -> String {
			String::from("erc20")
		}

		#[ink(message)]
		fn decimals(&self) -> u8 {
			0
		}

		#[ink(message)]
		fn total_supply(&self) -> Balance {
			self.total_supply
		}

		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> Balance {
			self.balance_of_impl(&owner)
		}

		#[ink(message)]
		fn transfer(&mut self, to: AccountId, value: Balance) -> Result<bool> {
			let from = self.env().caller();
			self.transfer_from_impl(&from, &to, value)
		}

		#[ink(message)]
		fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: Balance,
		) -> Result<bool> {
			let caller = self.env().caller();
			let allowance = self.allowance_impl(&from, &caller);
			if allowance < value {
				return Err(Error::AllowanceNotEnough);
			}

			self.transfer_from_impl(&from, &to, value)?;
			self.allowances.insert((&from, &caller), &(allowance - value));

			Ok(true)
		}

		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
			let owner = self.env().caller();
			self.allowances.insert((&owner, &spender), &value);
			self.env().emit_event(Approval { owner, spender, value });

			true
		}

		#[ink(message)]
		fn allowance(&mut self, owner: AccountId, spender: AccountId) -> Balance {
			self.allowance_impl(&owner, &spender)
		}

		#[ink(message)]
		fn burn(&mut self, value: Balance) -> Result<bool> {
			let owner = self.env().caller();
			let balance = self.balance_of_impl(&owner);
			if balance < value {
				return Err(Error::BalanceNotEnough);
			}

			self.balances.insert(&owner, &(balance - value));
			self.total_supply -= value;
			self.env().emit_event(Burn { owner, value, total_supply: self.total_supply });
			Ok(true)
		}

		#[ink(message)]
		fn freeze(&mut self, value: Balance) -> Result<bool> {
			let owner = self.env().caller();
			let balance = self.balance_of_impl(&owner);
			if balance < value {
				return Err(Error::BalanceNotEnough);
			}

			self.balances.insert(&owner, &(balance - value));
			self.freezes.insert(&owner, &value);
			self.env().emit_event(Freeze { owner, value });
			Ok(true)
		}

		#[ink(message)]
		fn unfreeze(&mut self, value: Balance) -> Result<bool> {
			let owner = self.env().caller();
			let freezed = self.freezes.get(&owner).unwrap_or_default();
			if freezed < value {
				return Err(Error::FrozenFundNotEnough);
			}

			let balance = self.balance_of_impl(&owner);
			self.freezes.insert(&owner, &(freezed - value));
			self.balances.insert(&owner, &(balance + value));
			self.env().emit_event(Unfreeze { owner, value });
			Ok(true)
		}
	}

	#[ink(impl)]
	impl Erc20 {
		#[inline]
		fn balance_of_impl(&self, owner: &AccountId) -> Balance {
			self.balances.get(owner).unwrap_or_default()
		}

		#[inline]
		fn allowance_impl(&mut self, owner: &AccountId, spender: &AccountId) -> Balance {
			self.allowances.get((owner, spender)).unwrap_or_default()
		}

		fn transfer_from_impl(
			&mut self,
			from: &AccountId,
			to: &AccountId,
			value: Balance,
		) -> Result<bool> {
			let from_balance = self.balance_of_impl(from);
			if from_balance < value {
				return Err(Error::BalanceNotEnough);
			}

			self.balances.insert(&from, &(from_balance - value));
			let to_balance = self.balance_of_impl(to);
			self.balances.insert(&to, &(to_balance + value));

			self.env().emit_event(Transfer { from: Some(*from), to: Some(*to), value });

			Ok(true)
		}
	}

	/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
	/// module and test functions are marked with a `#[test]` attribute.
	/// The below code is technically just normal Rust code.
	#[cfg(test)]
	mod tests {
		use ink::env::test::recorded_events;

		/// Imports all the definitions from the outer scope so we can use them here.
		use super::*;

		/// We test if the default constructor does its job.
		#[ink::test]
		fn new_works() {
			let _ = Erc20::new(1000);
			let emitted_events = recorded_events().collect::<Vec<_>>();
			assert_eq!(1, emitted_events.len());
		}

		/// We test a simple use case of our contract.
		#[ink::test]
		fn total_supply_works() {
			let erc20 = Erc20::new(1000);
			assert_eq!(erc20.total_supply(), 1000);
		}
	}
}
