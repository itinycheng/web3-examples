use std::{collections::HashMap, str::FromStr, vec};

use serde::Deserialize;
use serde_json::{Number, Value as JsonValue};
use web3::{
	contract::tokens::{Tokenizable, Tokenize},
	ethabi::Token,
	types::{H160, U256},
};

use crate::{error::Error, Result};

#[derive(Debug, Deserialize)]
pub struct ABI {
	pub constructor: ABIUnit,
	pub function: HashMap<String, ABIUnit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ABIUnit {
	pub r#type: UnitType,
	pub name: Option<String>,
	pub anonymous: Option<bool>,
	pub inputs: Option<Vec<Variable>>,
	pub outputs: Option<Vec<Variable>>,
	pub state_mutability: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Variable {
	pub name: String,
	pub r#type: String,
	pub internal_type: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum UnitType {
	EVENT,
	ERROR,
	CONSTRUCTOR,
	FUNCTION,
	RECEIVE,
	FALLBACK,
}

impl ABIUnit {
	pub fn to_params(&self, json: JsonValue) -> Result<Vec<Token>> {
		if self.inputs.is_none() {
			return Ok(vec![]);
		}

		let json_len = match &json {
			JsonValue::Array(arr) => arr.len(),
			JsonValue::Object(map) => map.len(),
			JsonValue::Null => 0,
			_ => 1,
		};
		let len_matched = match &self.inputs {
			Some(params) => params.len() == json_len,
			None => json_len == 0,
		};
		if !len_matched {
			return Err(Error::InvalidParam("params of abi parse failed".to_string()));
		}

		let mut params = Vec::with_capacity(json_len);
		for (idx, variable) in self.inputs.as_ref().unwrap().iter().enumerate() {
			match &json {
				JsonValue::Null => params.push(().into_tokens().into_token()),
				JsonValue::Bool(b) => params.push(b.into_token()),
				JsonValue::Number(num) => params.push(Self::num_to_token(num)?),
				JsonValue::String(string) => params.push(Self::string_to_token(string, variable)?),
				JsonValue::Array(arr) => {
					let value = arr.get(idx).unwrap();
					let token = Self::to_token(value, variable)?;
					params.push(token)
				}
				JsonValue::Object(_) => {
					return Err(Error::InvalidParam("Map type unsupported".to_string()));
				}
			};
		}

		Ok(params)
	}

	fn to_token(value: &JsonValue, variable: &Variable) -> Result<Token> {
		match value {
			JsonValue::Bool(b) => Ok(b.into_token()),
			JsonValue::Number(num) => Self::num_to_token(num),
			JsonValue::String(string) => Self::string_to_token(string, variable),
			JsonValue::Array(arr) => {
				let mut tokens = Vec::with_capacity(arr.len());
				for item in arr {
					tokens.push(Self::to_token(item, variable)?);
				}
				Ok(tokens.into_token())
			}
			_ => Err(Error::InvalidParam(format!(
				"Null and Map types are unsupported, data: {}",
				value
			))),
		}
	}

	fn num_to_token(num: &Number) -> Result<Token> {
		let err = || Error::InvalidParam("f64 is not supported".to_string());
		// TODO: a bug in ethabi.
		if num.is_i64() || num.is_u64() {
			num.as_u64().map(|n| n.into_token()).ok_or_else(err)
		} else {
			Err(Error::InvalidParam("f64 is not supported".to_string()))
		}
	}

	fn string_to_token(token_str: &String, variable: &Variable) -> Result<Token> {
		match &variable.r#type {
			v_type if v_type.starts_with("address") => token_str
				.parse::<H160>()
				.map(|token| token.into_token())
				.map_err(|e| Error::AnyError(e.into())),
			v_type if v_type.starts_with("uint256") => token_str
				.parse::<U256>()
				.map(|token| token.into_token())
				.map_err(|e| Error::AnyError(e.into())),
			_ => Err(Error::InvalidParam(format!(
				"convert {} to type: {} failed",
				token_str, variable.r#type
			))),
		}
	}
}

impl FromStr for ABI {
	type Err = crate::error::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let unit_list = serde_json::from_str::<Vec<ABIUnit>>(s)
			.map_err(|e| Self::Err::ABIParseError(e.to_string()))?;

		let mut type_map = unit_list.into_iter().fold(HashMap::new(), |mut map, unit| {
			map.entry(unit.r#type).or_insert_with(|| vec![unit]);
			map
		});
		let constructor = type_map
			.remove(&UnitType::CONSTRUCTOR)
			.map(|mut units| units.remove(0))
			.ok_or(Self::Err::ABIParseError("constructor not found".to_string()))?;

		let functions = type_map.remove(&UnitType::FUNCTION).map_or(HashMap::new(), |units| {
			units
				.into_iter()
				.map(|unit| (unit.name.clone().unwrap_or("".to_string()), unit))
				.collect::<HashMap<String, ABIUnit>>()
		});

		let abi = ABI { constructor: constructor, function: functions };
		Ok(abi)
	}
}

#[cfg(test)]
mod tests {
	use crate::contracts::ABI;

	#[test]
	fn test_create_abi() {
		let s = r#"[ {  "inputs": [{ "internalType": "address[]", "name": "_owners", "type": "address[]"},{ "internalType": "uint256", "name": "_numConfirmRequired", "type": "uint256"},{ "internalType": "bool", "name": "_anyDepositAllowed", "type": "bool"}  ],  "stateMutability": "nonpayable",  "type": "constructor" }]"#;
		let abi = s.parse::<ABI>();
		print!("{:?}", abi.unwrap());
	}
}
