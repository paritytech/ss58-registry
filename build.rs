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
//use rustfmt_nightly::Input

#[derive(Deserialize)]
struct Registry {
	#[serde(rename = "registry")]
	accounts: Vec<AccountType>,
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
		for account_type in &self.accounts {
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
		}
		Ok(())
	}
}

#[non_exhaustive]
#[derive(Deserialize, Debug, Clone, Copy)]
enum SignatureType {
	#[serde(rename = "Sr25519")]
	Sr25519,
	#[serde(rename = "Ed25519")]
	Ed25519,
	#[serde(rename = "secp256k1")]
	Secp256k1,
	#[serde(rename = "*25519")]
	Any25519,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AccountType {
	prefix: u16,
	network: String,
	display_name: String,
	/// If standard account is None then the network is reserved.
	standard_account: Option<SignatureType>,
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

	let mut accounts = registry.accounts;

	// Sort by name so that we can later binary search by network
	accounts.sort_by_key(|a| a.network.clone());

	// Variables to insert into quote template:
	let identifier: Vec<_> = accounts.iter().map(|r| format_ident!("{}", r.name())).collect();

	let count = accounts.len();
	let prefix: Vec<_> = accounts.iter().map(|r| r.prefix).collect();

	let name = accounts.iter().map(|r| r.network.clone());
	let desc = accounts.iter().map(|r| {
		if let Some(website) = &r.website {
			format!("{} - <{}>", r.display_name, website)
		} else {
			r.display_name.clone()
		}
	});

	let mut prefix_to_idx: Vec<_> = prefix.iter().enumerate().map(|(a, b)| (b, a)).collect();
	prefix_to_idx.sort_by_key(|(prefix, _)| *prefix);
	let prefix_to_idx = prefix_to_idx.iter().map(|(prefix, idx)| quote! { (#prefix, #idx) });

	let reserved_prefixes = accounts.iter().filter(|r| r.is_reserved()).map(|r| r.prefix);

	let mut ordered_prefixes = accounts.iter().map(|i| i.prefix).collect::<Vec<_>>();
	ordered_prefixes.sort_unstable();
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
		pub(crate) static ALL_SS58_ADDRESS_FORMATS: [Ss58AddressFormatRegistry; #count] = [
			 #(Ss58AddressFormatRegistry::#identifier),*,
		];

		/// Names of all address formats (Sorted by network)
		pub(crate) static ALL_SS58_ADDRESS_FORMAT_NAMES: [&str; #count] = [
			#(#name),*,
		];

		/// (Sorted) prefixes to index of ALL_SS58_ADDRESS_FORMATS
		pub(crate) static PREFIX_TO_INDEX: [(u16, usize); #count] = [
			#(#prefix_to_idx),*,
		];

		impl Ss58AddressFormat {
			/// Network/AddressType is reserved for future use.
			pub fn is_reserved(&self) -> bool {
				self.prefix() > 16384 || matches!(self.prefix(), #(#reserved_prefixes)|*)
			}

			/// A custom format is one that is not already known.
			pub fn is_custom(&self) -> bool {
				// A match is faster than bin search
				// as most hits will be in the first group.
				!matches!(self.prefix(), #(#prefix_starts..=#prefix_ends)|*)
			}
		}
	})
}

fn fmt(unformatted: String) -> Result<String, String> {
	use std::process::*;
	use std::io::{Write, copy};

	let cfg_path = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR exists");
	let mut cmd = Command::new("rustfmt");
	cmd.arg("--config-path").arg(&cfg_path).stdin(Stdio::piped()).stdout(Stdio::piped());

	let mut child = cmd.spawn().map_err(|e| format!("Cannot spawn rustfmt: {}", e))?;
	let mut child_stdin = child.stdin.take().expect("Set stdin to be a pipe. qed");
	let mut child_stdout = child.stdout.take().expect("Set stdout to be a pipe. qed");

	let mut output = Vec::with_capacity(unformatted.len() * 2);
	let stdin_handle = ::std::thread::spawn(move || {
		let res = child_stdin.write_all(unformatted.as_bytes());
		drop(res);
	});
	
	copy(&mut child_stdout, &mut output).map_err(|e| format!("Cannot read rustfmt output: {}", e))?;

	let status = child.wait().map_err(|e| format!("Child process rustfmt failed: {}", e))?;
	stdin_handle.join().expect(
		"The thread writing to rustfmt's stdin doesn't do \
		 anything that could panic",
	);
	match status.code() {
		Some(0) => Ok(()),
		Some(1) => Err("Rustfmt syntax errors".to_owned()),
		Some(2) => Err("Rustfmt parsing errors".to_owned()),
		Some(3) => Err("Rustfmt could not format some lines".to_owned()),
		_ => Err("Internal rustfmt error".to_owned()),
	}?;
	let output = String::from_utf8(output).map_err(|e| format!("Invalid output from rustfmt: {}", e))?;
	Ok(output)
}

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=ss58-registry.json");

	let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR should exist");

	let unformatted = match create_ss58_registry(include_str!("ss58-registry.json")) {
		Ok(result) => result.to_string(),
		Err(msg) => {
			eprintln!("failed to generate code from json: {}", &msg);
			std::process::exit(-1);
		},
	};

	let formatted = match fmt(unformatted) {
		Ok(s) => s,
		Err(msg) => {
			eprintln!("failed to format generated code: {}", &msg);
			std::process::exit(-1);
		}
	};

	let dest_path = Path::new(&out_dir).join("account_type_enum.rs");
	if let Err(err) = fs::write(&dest_path, formatted) {
		eprintln!("failed to write generated code to {}: {}", &dest_path.display(), err);
		std::process::exit(-1);
	}


}
