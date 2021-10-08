// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! List of wellknown SS58 account types as an enum.
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::Path};
use unicode_xid::UnicodeXID;

#[derive(Deserialize)]
struct Registry {
	registry: Vec<AccountType>,
}

fn is_valid_rust_identifier(id: &str) -> Result<(), String> {
	if let Some(ch) = id.chars().next() {
		if ch.is_xid_start() {
			for ch in id.chars().skip(1) {
				if !ch.is_xid_continue() {
					return Err(format!("Invalid char `{}` in `{}`", ch, id))
				}
			}
			Ok(())
		} else {
			Err(format!("`{}` starts with `{}` which is not valid at the start", id, ch))
		}
	} else {
		Err("empty identifier".into())
	}
}

impl Registry {
	pub fn is_valid(&self) -> Result<(), String> {
		let mut used_prefixes = HashMap::<u16, AccountType>::new();
		let mut used_networks = HashMap::<String, AccountType>::new();
		for account_type in &self.registry {
			if let Some(clash) = used_prefixes.insert(account_type.prefix, (*account_type).clone())
			{
				return Err(format!(
                    "prefixes must be unique but this account's prefix:\n{:#?}\nclashed with\n{:#?}",
                    account_type,
                    clash
                ));
			}
			if let Some(clash) = used_networks.insert(account_type.name(), account_type.clone()) {
				return Err(format!(
                    "networks must be unique but this account's network:\n{:#?}\nclashed with\n:{:#?}",
                    account_type,
                    clash
                ));
			}
			if account_type.network.is_empty() {
				return Err("network is mandatory.".into())
			}

			if let Err(err) = is_valid_rust_identifier(&account_type.name()) {
				return Err(format!("network not valid: {} for {:#?}", err, account_type))
			}
			if account_type.decimals.len() != account_type.symbols.len() {
				return Err(format!(
					"decimals must be specified for each symbol: {:?}",
					account_type
				))
			}
			if let Some(sig_type) = &account_type.standard_account {
				match sig_type.as_str() {
                    "Sr25519"| "Ed25519" | "secp256k1" | "*25519" => {},
                    _ => {
                        return Err(format!("Unknown sig type in standardAccount: expected one of Sr25519, Ed25519, secp256k1, *25519: {:?}", account_type))
                    }
                }
			}
		}
		Ok(())
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AccountType {
	prefix: u16,
	network: String,
	display_name: String,
	/// If standard account is None then the network is reserved.
	standard_account: Option<String>,
	symbols: Vec<String>,
	decimals: Vec<u8>,
	website: Option<String>,
}

impl AccountType {
	fn name(&self) -> String {
		format!("{}Account", inflector::cases::pascalcase::to_pascal_case(&self.network))
	}

	fn is_reserved(&self) -> bool {
		self.standard_account.is_none()
	}
}

fn consecutive_runs(data: &[u16]) -> (Vec<u16>, Vec<u16>) {
	let mut slice_start = 0_u16;
	let (mut starts, mut ends) = (Vec::new(), Vec::new());
	for i in 1..data.len() {
		if data[i - 1] + 1 != data[i] {
			starts.push(slice_start);
			ends.push(data[(i - 1)]);
			slice_start = data[i];
		}
	}
	if !data.is_empty() {
		starts.push(slice_start);
		ends.push(data[data.len() - 1]);
	}
	(starts, ends)
}

fn create_ss58_registry(json: &str) -> Result<proc_macro2::TokenStream, String> {
	let registry: Registry =
		serde_json::from_str(json).map_err(|e| format!("json parsing error: {}", e))?;
	registry.is_valid()?;

	let mut registry = registry.registry;

	let mut ordered_prefixes = registry.iter().map(|i| i.prefix).collect::<Vec<_>>();
	ordered_prefixes.sort();

	// Sort by name so that we can later binary search by network
	registry.sort_by_key(|a| a.network.clone());

	// Variables to insert into quote template:
	let identifier: Vec<_> = registry.iter().map(|r| format_ident!("{}", r.name())).collect();

	let reserved_prefixes = registry.iter().filter(|r| r.is_reserved()).map(|r| r.prefix);

	let count = registry.len();
	let prefix: Vec<_> = registry.iter().map(|r| r.prefix).collect();

	let mut prefix_to_idx: Vec<_> = prefix.iter().enumerate().map(|(a, b)| (b, a)).collect();
	prefix_to_idx.sort_by_key(|(prefix, _)| *prefix);
	let prefix_to_idx = prefix_to_idx.iter().map(|(prefix, idx)| quote! { (#prefix, #idx) });

	let name = registry.iter().map(|r| r.network.clone());
	let desc = registry.iter().map(|r| {
		if let Some(website) = &r.website {
			format!("{} - <{}>", r.display_name, website)
		} else {
			r.display_name.clone()
		}
	});

	let (prefix_starts, prefix_ends) = consecutive_runs(ordered_prefixes.as_slice());

	Ok(quote! {
		/// A known address (sub)format/network ID for SS58.
		#[non_exhaustive]
		#[repr(u16)]
		#[derive(Copy, Clone, PartialEq, Eq, Debug)]
		pub enum Ss58AddressFormatRegistry {
			#(#[doc = #desc] #identifier = #prefix),*,
		}

		/// All non-custom address formats (Sorted by network)
		static ALL_SS58_ADDRESS_FORMATS: [Ss58AddressFormatRegistry; #count] = [
			 #(Ss58AddressFormatRegistry::#identifier),*,
		];

		/// Names of all address formats (Sorted by network)
		static ALL_SS58_ADDRESS_FORMAT_NAMES: [&str; #count] = [
			#(#name),*,
		];

		/// (Sorted) prefixes to index of ALL_SS58_ADDRESS_FORMATS
		static PREFIX_TO_INDEX: [(u16, usize); #count] = [
			#(#prefix_to_idx),*,
		];

		impl Ss58AddressFormat {
			/// Network/AddressType is reserved for future use.
			pub fn is_reserved(&self) -> bool {
				self.prefix > 16384 || matches!(self.prefix, #(#reserved_prefixes)|*)
			}

			/// A custom format is one that is not already known.
			pub fn is_custom(&self) -> bool {
				// A match is faster than bin search
				// as most hits will be in the first group.
				!matches!(self.prefix, #(#prefix_starts..=#prefix_ends)|*)
			}
		}
	})
}

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=ss58-registry.json");

	let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR should exist");

	let result = match create_ss58_registry(include_str!("ss58-registry.json")) {
		Ok(result) => result.to_string(),
		Err(msg) => {
			eprintln!("failed to generate code from json: {}", &msg);
			std::process::exit(-1);
		},
	};

	let dest_path = Path::new(&out_dir).join("account_type_enum.rs");
	if let Err(err) = fs::write(&dest_path, result) {
		eprintln!("failed to write generated code to {}: {}", &dest_path.display(), err);
		std::process::exit(-1);
	}
}
