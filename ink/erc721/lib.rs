#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	NotOwner,
	NotApproved,
	TokenNotExists,
	TokenIsOwned,
	NotAllowed,
	OwnerNotFound,
	InvalidOwnerCount,
	OneTokenMinted,
}

#[ink::contract]
mod erc721 {
	use ink::prelude::string::String;
	use ink::storage::Mapping;

	use crate::Error;

	pub type TokenId = String;
	pub type Result<T> = core::result::Result<T, Error>;

	#[ink(event)]
	pub struct Transfer {
		#[ink(topic)]
		from: AccountId,
		#[ink(topic)]
		to: AccountId,
		#[ink(topic)]
		token_id: TokenId,
	}

	#[ink(event)]
	pub struct Approval {
		#[ink(topic)]
		from: AccountId,
		#[ink(topic)]
		to: AccountId,
		#[ink(topic)]
		token_id: TokenId,
	}

	#[ink(event)]
	pub struct ApprovalForAll {
		#[ink(topic)]
		owner: AccountId,
		#[ink(topic)]
		operator: AccountId,
		approved: bool,
	}

	#[derive(Default)]
	#[ink(storage)]
	pub struct Erc721 {
		token_owner: Mapping<TokenId, AccountId>,
		token_approvals: Mapping<TokenId, AccountId>,
		owner_token_count: Mapping<AccountId, Balance>,
		operator_approvals: Mapping<(AccountId, AccountId), ()>,
	}

	impl Erc721 {
		#[ink(constructor)]
		pub fn new() -> Self {
			Self::default()
		}

		#[ink(message)]
		pub fn name(&self) -> String {
			String::from("erc721-faker")
		}

		#[ink(message)]
		pub fn symbol(&self) -> String {
			String::from("faker")
		}

		#[ink(message)]
		pub fn token_uri(&self, token_id: TokenId) -> String {
			String::from("ipfs:") + &token_id
		}

		#[ink(message)]
		pub fn balance_of(&self, owner: AccountId) -> Balance {
			self.owner_token_count.get(owner).unwrap_or_default()
		}

		#[ink(message)]
		pub fn owner_of(&self, token_id: TokenId) -> Option<AccountId> {
			self.token_owner.get(token_id)
		}

		#[ink(message)]
		pub fn get_approved(&self, token_id: TokenId) -> Option<AccountId> {
			self.token_approvals.get(token_id)
		}

		#[ink(message)]
		pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
			self.operator_approvals.contains((&owner, &operator))
		}

		#[ink(message, payable)]
		pub fn transfer(&mut self, to: AccountId, token_id: TokenId) -> Result<()> {
			let from = self.env().caller();
			self.transfer_token_impl(from, to, token_id)
		}

		#[ink(message, payable)]
		pub fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			token_id: TokenId,
		) -> Result<()> {
			self.transfer_token_impl(from, to, token_id)
		}

		#[ink(message, payable)]
		pub fn approve(&mut self, to: AccountId, token_id: TokenId) -> Result<()> {
			let caller = self.env().caller();
			let owner_option = self.owner_of_impl(&token_id);
			if Some(&caller) == owner_option.as_ref()
				|| self.operator_approvals.contains((&owner_option.unwrap(), &caller))
			{
				return Err(Error::NotAllowed);
			}

			let owner = owner_option.unwrap();
			if self.token_approvals.contains(&token_id) {
				return Err(Error::NotAllowed);
			}

			self.token_approvals.insert(&token_id, &to);

			self.env().emit_event(Approval { from: owner, to, token_id });
			Ok(())
		}

		#[ink(message)]
		pub fn set_approval_for_all(&mut self, operator: AccountId, approved: bool) -> Result<()> {
			let caller = self.env().caller();
			if caller == operator {
				return Err(Error::NotAllowed);
			}

			if approved {
				self.operator_approvals.insert((&caller, &operator), &());
			} else {
				self.operator_approvals.remove((&caller, &operator))
			}

			self.env().emit_event(ApprovalForAll { owner: caller, operator, approved });
			Ok(())
		}

		#[ink(message)]
		pub fn mint(&mut self, token_id: TokenId) -> Result<()> {
			let caller = self.env().caller();
			let Self { token_owner, owner_token_count, .. } = self;
			if token_owner.contains(&token_id) {
				return Err(Error::TokenIsOwned);
			}

			if owner_token_count.get(&caller) > Some(0) {
				return Err(Error::OneTokenMinted);
			}

			token_owner.insert(&token_id, &caller);
			owner_token_count.insert(&caller, &1);
			self.env().emit_event(Transfer {
				from: AccountId::from([0; 32]),
				to: caller,
				token_id,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn burn(&mut self, token_id: TokenId) -> Result<()> {
			let caller = self.env().caller();
			let Self { token_owner, owner_token_count, .. } = self;
			let owner = token_owner.get(&token_id).ok_or(Error::OwnerNotFound)?;
			if owner != caller {
				return Err(Error::NotOwner);
			}

			let back_hole = AccountId::from([0x0; 32]);
			token_owner.insert(&token_id, &back_hole);
			let count = owner_token_count.get(&back_hole).map(|c| c + 1).unwrap_or_default();
			owner_token_count.insert(&back_hole, &count);
			self.env().emit_event(Transfer { from: caller, to: back_hole, token_id });
			Ok(())
		}

		// -------------------------------------------
		// ----------- internal functions ------------
		// -------------------------------------------

		pub fn transfer_token_impl(
			&mut self,
			from: AccountId,
			to: AccountId,
			token_id: TokenId,
		) -> Result<()> {
			if !self.exists(&token_id) {
				return Err(Error::TokenNotExists);
			}

			if !self.is_owner_or_approved(&from, &token_id) {
				return Err(Error::NotOwner);
			}

			self.token_approvals.remove(&token_id);
			self.remove_token_from_owner(&token_id)?;
			self.add_token_to(&to, &token_id)?;

			self.env().emit_event(Transfer { from, to, token_id });
			Ok(())
		}

		pub fn add_token_to(&mut self, to: &AccountId, token_id: &TokenId) -> Result<()> {
			let Self { token_owner, owner_token_count, .. } = self;
			if token_owner.contains(token_id) {
				return Err(Error::TokenIsOwned);
			}

			token_owner.insert(token_id, to);
			let count = owner_token_count.get(to).map(|c| c + 1).unwrap_or(1);
			owner_token_count.insert(&to, &count);
			Ok(())
		}

		pub fn remove_token_from_owner(&mut self, token_id: &TokenId) -> Result<()> {
			let Self { token_owner, owner_token_count, .. } = self;
			if !token_owner.contains(token_id) {
				return Err(Error::TokenNotExists);
			}

			let owner = token_owner.get(token_id).ok_or(Error::OwnerNotFound)?;
			let count =
				owner_token_count.get(&owner).map(|c| c - 1).ok_or(Error::InvalidOwnerCount)?;
			owner_token_count.insert(&owner, &count);
			token_owner.remove(token_id);
			Ok(())
		}

		pub fn is_owner_or_approved(&self, from: &AccountId, token_id: &TokenId) -> bool {
			let owner = self.token_owner.get(token_id);
			let approved_operator = self.token_approvals.get(&token_id);
			owner.as_ref() == Some(from)
				|| approved_operator.as_ref() == Some(from)
				|| self.operator_approvals.contains((&owner.unwrap(), &from))
		}

		#[inline]
		pub fn owner_of_impl(&self, token_id: &TokenId) -> Option<AccountId> {
			self.token_owner.get(token_id)
		}

		#[inline]
		pub fn exists(&self, token_id: &TokenId) -> bool {
			self.token_owner.contains(token_id)
		}
	}

	/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
	/// module and test functions are marked with a `#[test]` attribute.
	/// The below code is technically just normal Rust code.
	#[cfg(test)]
	mod tests {

		/// We test if the default constructor does its job.
		#[ink::test]
		fn test_options() {
			assert_eq!(false, None > Some(0));
			assert_eq!(true, Some(1) > Some(0));
		}

		/// We test a simple use case of our contract.
		#[ink::test]
		fn it_works() {}
	}
}
