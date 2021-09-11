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

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde::{self, Deserialize};
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Deserialize)]
struct Registry {
    registry: Vec<AccountType>,
}

impl Registry {
    pub fn is_valid(&self) -> Result<(), String> {
        let unique_ids: HashSet<u16> = self.registry.iter().map(|r| r.prefix).collect();
        if unique_ids.len() != self.registry.len() {
            return Err("prefixes must be unique.".to_owned());
        }

        let unreserved_networks: Vec<String> = self
            .registry
            .iter()
            .filter_map(|r| r.network.clone())
            .collect();
        let unique_networks: HashSet<&String> = HashSet::from_iter(unreserved_networks.iter());
        if unique_networks.len() != unreserved_networks.len() {
            return Err("networks must be unique.".to_owned());
        }

        for account_type in &self.registry {
            if let Some(network) = &account_type.network {
                if network.chars().any(|c| c.is_whitespace()) {
                    return Err(format!(
                        "network can not have whitespace in: {:?}",
                        account_type
                    ));
                }
            }

            if let Some(symbols) = &account_type.symbols {
                if account_type
                    .decimals
                    .as_ref()
                    .filter(|decimals| symbols.len() == decimals.len())
                    .is_none()
                {
                    return Err(format!(
                        "decimals must be specified for each symbol: {:?}",
                        account_type
                    ));
                }
            } else if account_type.decimals.is_some() {
                return Err(format!(
                    "decimals can't be specified without symbols: {:?}",
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
struct AccountType {
    prefix: u16,
    network: Option<String>,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "standardAccount")]
    standard_account: Option<String>,
    website: Option<String>,
    symbols: Option<Vec<String>>,
    decimals: Option<Vec<u8>>,
}

impl AccountType {
    fn name(&self) -> String {
        let mut name = self.network.clone().unwrap_or(
            if let Some(standard_account) = &self.standard_account {
                format!("Bare{}", standard_account)
            } else {
                let name = self
                    .network
                    .as_ref()
                    .expect("network should not be empty if no account specified")
                    .to_string();

                assert!(
                    name.starts_with("reserved"),
                    "If no account specified, network should start `reserved` not:{}",
                    name
                );
                name
            },
        );
        let is_reserved = name.starts_with("reserved");
        if name.ends_with("net") {
            name.truncate(name.len() - 3);
        }
        // Capitalise
        name.get_mut(0..1)
            .expect("name should not be empty")
            .make_ascii_uppercase();

        let postfix = if is_reserved { "" } else { "Account" };
        format!("{}{}", rust_valid_id(name), postfix)
    }
}

fn rust_valid_id(name: String) -> String {
    // TODO find function that already does this. `-` are excluded in particular.
    name.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Creates the Ss58AddressFormat enum from the ss58-registry.json file
#[proc_macro]
pub fn ss58_registry(input: TokenStream) -> TokenStream {
    assert!(input.is_empty(), "No arguments are expected");

    match create_ss58_registry(include_str!("ss58-registry.json")) {
        Ok(result) => result,
        Err(msg) => panic!("{}", msg),
    }
}

fn create_ss58_registry(json: &str) -> Result<TokenStream, String> {
    let registry: Registry = serde_json::from_str(json).expect("valid json file");

    registry.is_valid()?;

    let identifier: Vec<_> = registry
        .registry
        .iter()
        .map(|r| format_ident!("{}", r.name()))
        .collect();

    let reserved_identifiers: Vec<_> = registry
        .registry
        .iter()
        .filter(|r| r.network.as_ref().filter(|r|r.starts_with("reserved")).is_some())
        .map(|r| format_ident!("{}", r.name()))
        .collect();

    let reserved_numbers: Vec<_> = registry
        .registry
        .iter()
        .filter(|r| r.network.as_ref().filter(|r|r.starts_with("reserved")).is_some())
        .map(|r| r.prefix)
        .collect();

    let number: Vec<_> = registry.registry.iter().map(|r| r.prefix).collect();
    let count = registry.registry.len();
    let name: Vec<_> = registry
        .registry
        .iter()
        .map(|r| r.network.clone().unwrap_or_else(|| "Bare".to_string()))
        .collect();
    let desc = registry.registry.iter().map(|r| {
        if let Some(website) = &r.website {
            format!("{} - <{}>", r.display_name, website)
        } else {
            r.display_name.clone()
        }
    });

    let output = quote! {
        /// Default prefix number
        #[cfg(feature = "std")]
        static DEFAULT_VERSION: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(42 /*substrate*/);

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
                    #(#reserved_identifiers => true),*,
                    Ss58AddressFormat::Custom(prefix) => {
                        match prefix {
                            #(#reserved_numbers => true),*,
                            _ => false,
                        }
                    },
                    _ => false,
                }
            }

            /// Is this address format the current default?
            #[cfg(feature = "std")]
            pub fn is_default(&self) -> bool {
                self == &Self::default()
            }
        }

        impl From<u8> for Ss58AddressFormat {
            fn from(x: u8) -> Ss58AddressFormat {
                Ss58AddressFormat::from(x as u16)
            }
        }

        impl From<Ss58AddressFormat> for u16 {
            fn from(x: Ss58AddressFormat) -> u16 {
                match x {
                    #(Ss58AddressFormat::#identifier => #number),*,
                    Ss58AddressFormat::Custom(n) => n,
                }
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
        impl Default for Ss58AddressFormat {
            fn default() -> Self {
                DEFAULT_VERSION.load(core::sync::atomic::Ordering::Relaxed).into()
            }
        }

        /// Set the default "version" (actually, this is a bit of a misnomer and the version byte is
        /// typically used not just to encode format/version but also network identity) that is used for
        /// encoding and decoding SS58 addresses.
        #[cfg(feature = "std")]
        pub fn set_default_ss58_version(new_default: Ss58AddressFormat) {
            let prefix : u16 = new_default.into();
            DEFAULT_VERSION.store(prefix, core::sync::atomic::Ordering::Relaxed);
        }

        #[cfg(feature = "std")]
        impl From<Ss58AddressFormat> for String {
            fn from(x: Ss58AddressFormat) -> String {
                x.to_string()
            }
        }
    };

    Ok(output.into())
}
