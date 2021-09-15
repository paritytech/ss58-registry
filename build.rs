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
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Registry {
    registry: Vec<AccountType>,
}

impl Registry {
    pub fn is_valid(&self) -> Result<(), String> {
        let mut used_prefixes = HashSet::<u16>::new();
        let mut used_networks = HashSet::<String>::new();

        for account_type in &self.registry {
            if !used_prefixes.insert(account_type.prefix) {
                return Err(format!(
                    "prefixes must be unique but this account's prefix clashes: {:#?}.",
                    account_type
                ));
            }
            if !used_networks.insert(account_type.network.clone()) {
                return Err(format!(
                    "networks must be unique but this account's network clashes: {:#?}.",
                    account_type
                ));
            }
            if account_type.network.chars().any(|c| c.is_whitespace()) {
                return Err(format!(
                    "network can not have whitespace in: {:?}",
                    account_type
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

#[derive(Deserialize, Debug)]
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

    let registry = registry.registry;

    // Variables to insert into quote template:
    let identifier: Vec<_> = registry
        .iter()
        .map(|r| format_ident!("{}", r.name()))
        .collect();

    let reserved_identifiers = registry
        .iter()
        .filter(|r| r.is_reserved())
        .map(|r| format_ident!("{}", r.name()));

    let reserved_numbers = registry
        .iter()
        .filter(|r| r.is_reserved())
        .map(|r| r.prefix);

    let count = registry.len();
    let number: Vec<_> = registry.iter().map(|r| r.prefix).collect();
    let name: Vec<_> = registry.iter().map(|r| r.network.clone()).collect();
    let desc = registry.iter().map(|r| {
        if let Some(website) = &r.website {
            format!("{} - <{}>", r.display_name, website)
        } else {
            r.display_name.clone()
        }
    });

    Ok(quote! {
        use sp_debug_derive::RuntimeDebug;

        /// A known address (sub)format/network ID for SS58.
        #[derive(Copy, Clone, PartialEq, Eq, crate::RuntimeDebug)]
        pub enum Ss58AddressFormat {
            #(#[doc = #desc] #identifier),*,
            /// Use a manually provided numeric value as a standard identifier
            Custom(u16),
        }

        /// Display the name of the address format (not the description).
        #[cfg(feature = "std")]
        impl std::fmt::Display for Ss58AddressFormat {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(
                        Ss58AddressFormat::#identifier => write!(f, "{}", #name),
                    )*
                    Ss58AddressFormat::Custom(x) => write!(f, "{}", x),
                }

            }
        }

        /// All non-custom address formats.
        static ALL_SS58_ADDRESS_FORMATS: [Ss58AddressFormat; #count] = [
             #(Ss58AddressFormat::#identifier),*,
        ];

        /// An enumeration of unique networks.
        /// Some are reserved.
        impl Ss58AddressFormat {
            /// names of all address formats
            pub fn all_names() -> &'static [&'static str] {
                &[
                    #(#name),*,
                ]
            }
            /// All known address formats.
            pub fn all() -> &'static [Ss58AddressFormat] {
                &ALL_SS58_ADDRESS_FORMATS
            }

            /// Whether the address is custom.
            pub fn is_custom(&self) -> bool {
                matches!(self, Self::Custom(_))
            }

            /// Network/AddressType is reserved for future use.
            pub fn is_reserved(&self) -> bool {
                match self {
                    #(Ss58AddressFormat::#reserved_identifiers)|* => true,
                    Ss58AddressFormat::Custom(prefix) => matches!(prefix, #(#reserved_numbers)|*),
                    _ => false,
                }
            }
        }

        impl From<u8> for Ss58AddressFormat {
            fn from(x: u8) -> Ss58AddressFormat {
                Ss58AddressFormat::from(x as u16)
            }
        }

        impl From<Ss58AddressFormat> for u16 {
            fn from(x: Ss58AddressFormat) -> u16 {
                from_address_format(x)
            }
        }

        /// const function to convert Ss58AddressFormat to u16
        pub const fn from_address_format(x: Ss58AddressFormat) -> u16 {
            match x {
                #(Ss58AddressFormat::#identifier => #number),*,
                Ss58AddressFormat::Custom(n) => n,
            }
        }

        impl From<u16> for Ss58AddressFormat {
            fn from(x: u16) -> Ss58AddressFormat {
                match x {
                    #(#number => Ss58AddressFormat::#identifier),*,
                    _ => Ss58AddressFormat::Custom(x),
                }
            }
        }

        /// Error encountered while parsing `Ss58AddressFormat` from &'_ str
        /// unit struct for now.
        #[derive(Copy, Clone, PartialEq, Eq, crate::RuntimeDebug)]
        pub struct ParseError;

        impl<'a> core::convert::TryFrom<&'a str> for Ss58AddressFormat {
            type Error = ParseError;

            fn try_from(x: &'a str) -> Result<Ss58AddressFormat, Self::Error> {
                match x {
                    #(#name => Ok(Ss58AddressFormat::#identifier)),*,
                    a => a.parse::<u16>().map(Ss58AddressFormat::Custom).map_err(|_| ParseError),
                }
            }
        }

        #[cfg(feature = "std")]
        impl std::str::FromStr for Ss58AddressFormat {
            type Err = ParseError;

            fn from_str(data: &str) -> Result<Self, Self::Err> {
                core::convert::TryFrom::try_from(data)
            }
        }

        #[cfg(feature = "std")]
        impl std::fmt::Display for ParseError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "failed to parse network value as u16")
            }
        }

        #[cfg(feature = "std")]
        impl From<Ss58AddressFormat> for String {
            fn from(x: Ss58AddressFormat) -> String {
                x.to_string()
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
    fs::write(&dest_path, result).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}