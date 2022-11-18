#![cfg_attr(not(feature = "std"), no_std)]

use scale::{Encode, Decode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	BalanceNotEnough,
	AllowanceNotEnough,
	FrozenFundNotEnough,
}

#[ink::contract]
mod call_erc20 {
	use ink::env::{
		call::{build_call, Call, ExecutionInput, Selector},
		CallFlags,
	};

	use crate::Error;

	pub type Result<T> = core::result::Result<T, Error>;

	#[ink(storage)]
	pub struct CallErc20 {
		callee: AccountId,
	}

	impl CallErc20 {
		#[ink(constructor)]
		pub fn new(erc20: AccountId) -> Self {
			Self { callee: erc20 }
		}

		#[ink(message)]
		pub fn balance_of(&self, owner: AccountId) -> Balance {
			build_call::<<Self as ::ink::reflect::ContractEnv>::Env>()
				.call_type(Call::new().callee(self.callee))
				.gas_limit(0)
				.transferred_value(0)
				.call_flags(CallFlags::default())
				.exec_input(
					ExecutionInput::new(Selector::new([0x93, 0x3a, 0xe3, 0xc8])).push_arg(&owner),
				)
				.returns::<Balance>()
				.fire()
				.unwrap()
		}

		#[ink(message, payable)]
		pub fn transfer_from(&self, from: AccountId, to: AccountId, value: Balance) -> Result<bool> {
			build_call::<<Self as ::ink::reflect::ContractEnv>::Env>()
				.call_type(Call::new().callee(self.callee))
				.gas_limit(0)
				.transferred_value(0)
				.call_flags(CallFlags::default())
				.exec_input(
					ExecutionInput::new(Selector::new(0x839f0263_u32.to_be_bytes()))
						.push_arg(&from)
						.push_arg(&to)
						.push_arg(&value),
				)
				.returns::<Result<bool>>()
				.fire()
				.unwrap()
		}
	}

	#[cfg(test)]
	mod tests {

		#[ink::test]
		fn it_works() {
			let num = u32::from_ne_bytes([1, 0, 0, 0]);
			println!("{}", num);
			assert_eq!(32, 0x00000020);

			let arr = 0x839f0263u32.to_be_bytes();
			assert_eq!(arr, [0x83, 0x9f, 0x02, 0x63])
		}
	}
}
