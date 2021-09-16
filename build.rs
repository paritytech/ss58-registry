// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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
#![deny(missing_docs)]
use quote::{format_ident, quote};
use serde::{self, Deserialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
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
                    return Err(format!("Invalid char `{}` in `{}`", ch, id));
                }
            }
            Ok(())
        } else {
            Err(format!(
                "`{}` starts with `{}` which is not valid at the start",
                id, ch
            ))
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
                return Err("network is mandatory.".into());
            }

            if let Err(err) = is_valid_rust_identifier(&account_type.name()) {
                return Err(format!(
                    "network not valid: {} for {:#?}",
                    err, account_type
                ));
            }
            if account_type.decimals.len() != account_type.symbols.len() {
                return Err(format!(
                    "decimals must be specified for each symbol: {:?}",
                    account_type
                ));
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
        format!(
            "{}Account",
            inflector::cases::pascalcase::to_pascal_case(&self.network)
        )
    }

    fn is_reserved(&self) -> bool {
        self.standard_account.is_none()
    }
}

fn create_ss58_registry(json: &str) -> Result<proc_macro2::TokenStream, String> {
    let registry: Registry = serde_json::from_str(json).expect("valid json file");
    registry.is_valid()?;

    let mut registry = registry.registry;
    // The names are assumed to be sorted.
    registry.sort_by_key(|a| a.name());

    // Variables to insert into quote template:
    let identifier: Vec<_> = registry
        .iter()
        .map(|r| format_ident!("{}", r.name()))
        .collect();

    let reserved_identifiers = registry
        .iter()
        .filter(|r| r.is_reserved())
        .map(|r| format_ident!("{}", r.name()));

    let count = registry.len();
    let number: Vec<_> = registry.iter().map(|r| r.prefix).collect();
    let enumeration: Vec<_> = (0..registry.len()).collect();

    let mut prefix_to_idx: Vec<_> = number.iter().zip(enumeration).collect();
    prefix_to_idx.sort_by_key(|(prefix, _)| *prefix);
    let prefix_to_idx = prefix_to_idx
        .iter()
        .map(|(prefix, idx)| quote! { (#prefix, #idx) });

    let name: Vec<_> = registry.iter().map(|r| r.network.clone()).collect();
    let desc = registry.iter().map(|r| {
        if let Some(website) = &r.website {
            format!("{} - <{}>", r.display_name, website)
        } else {
            r.display_name.clone()
        }
    });

    Ok(quote! {

        /// A known address (sub)format/network ID for SS58.
        #[non_exhaustive]
        #[repr(u16)]
        #[derive(Copy, Clone, PartialEq, Eq, crate::RuntimeDebug)]
        pub enum KnownSs58AddressFormat {
            #(#[doc = #desc] #identifier = #number),*,
        }

        /// All non-custom address formats (Sorted by name)
        static ALL_SS58_ADDRESS_FORMATS: [KnownSs58AddressFormat; #count] = [
             #(KnownSs58AddressFormat::#identifier),*,
        ];

        /// Names of all address formats (Sorted by name)
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
                if let Ok(known) = KnownSs58AddressFormat::try_from(*self) {
                    matches!(known, #(KnownSs58AddressFormat::#reserved_identifiers)|*)
                } else {
                    false
                }
            }
        }
    })
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR should exist");

    let result: String = match create_ss58_registry(include_str!("ss58-registry.json")) {
        Ok(result) => result.to_string(),
        Err(msg) => panic!("{}", msg),
    };

    let dest_path = Path::new(&out_dir).join("account_type_enum.rs");
    fs::write(&dest_path, result).unwrap_or_else(|_| panic!("failed to write to {:?}", &dest_path));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ss58-registry.json");
}
