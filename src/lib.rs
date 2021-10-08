// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0
//
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
#![warn(missing_docs)]

//! List of well-known SS58 account types as an enum.
use core::convert::TryFrom;

include!(concat!(env!("OUT_DIR"), "/account_type_enum.rs"));

/// Error encountered while parsing `Ss58AddressFormat` from &'_ str
/// unit struct for now.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ParseError;

/// A custom address format. See also [`Ss58AddressFormatRegistry`]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ss58AddressFormat {
	prefix: u16,
}

/// An enumeration of unique networks.
/// Some are reserved.
impl Ss58AddressFormat {
	/// Custom constructor
	#[inline]
	pub fn custom(prefix: u16) -> Self {
		Ss58AddressFormat { prefix }
	}

	/// names of all address formats
	pub fn all_names() -> &'static [&'static str] {
		&ALL_SS58_ADDRESS_FORMAT_NAMES
	}

	/// All known address formats.
	pub fn all() -> &'static [Ss58AddressFormatRegistry] {
		&ALL_SS58_ADDRESS_FORMATS
	}
}

/// Display the name of the address format (not the description).
#[cfg(feature = "std")]
impl std::fmt::Display for Ss58AddressFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Ok(lookup) =
			PREFIX_TO_INDEX.binary_search_by_key(&self.prefix, |(prefix, _)| *prefix)
		{
			let (_, idx) = PREFIX_TO_INDEX[lookup];
			write!(f, "{}", ALL_SS58_ADDRESS_FORMAT_NAMES[idx])
		} else {
			write!(f, "{}", self.prefix)
		}
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for Ss58AddressFormatRegistry {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let lookup = PREFIX_TO_INDEX
			.binary_search_by_key(&from_known_address_format(*self), |(prefix, _)| *prefix)
			.expect("always be found");
		let (_, idx) = PREFIX_TO_INDEX[lookup];
		write!(f, "{}", ALL_SS58_ADDRESS_FORMAT_NAMES[idx])
	}
}

impl TryFrom<Ss58AddressFormat> for Ss58AddressFormatRegistry {
	type Error = ParseError;

	fn try_from(x: Ss58AddressFormat) -> Result<Ss58AddressFormatRegistry, ParseError> {
		PREFIX_TO_INDEX
			.binary_search_by_key(&x.prefix, |(prefix, _)| *prefix)
			.map(|lookup| {
				let (_, idx) = PREFIX_TO_INDEX[lookup];
				ALL_SS58_ADDRESS_FORMATS[idx]
			})
			.map_err(|_| ParseError)
	}
}

/// const function to convert [`Ss58AddressFormat`] to u16
pub const fn from_address_format(x: Ss58AddressFormat) -> u16 {
	x.prefix
}

/// const function to convert [`Ss58AddressFormat`] to u16
pub const fn from_known_address_format(x: Ss58AddressFormatRegistry) -> u16 {
	x as u16
}

impl From<Ss58AddressFormatRegistry> for Ss58AddressFormat {
	fn from(x: Ss58AddressFormatRegistry) -> Ss58AddressFormat {
		Ss58AddressFormat { prefix: x as u16 }
	}
}

impl From<u8> for Ss58AddressFormat {
	#[inline]
	fn from(x: u8) -> Ss58AddressFormat {
		Ss58AddressFormat::from(u16::from(x))
	}
}

impl From<Ss58AddressFormat> for u16 {
	#[inline]
	fn from(x: Ss58AddressFormat) -> u16 {
		from_address_format(x)
	}
}

impl From<u16> for Ss58AddressFormat {
	#[inline]
	fn from(prefix: u16) -> Ss58AddressFormat {
		Ss58AddressFormat { prefix }
	}
}

impl<'a> TryFrom<&'a str> for Ss58AddressFormat {
	type Error = ParseError;

	fn try_from(x: &'a str) -> Result<Ss58AddressFormat, Self::Error> {
		Ss58AddressFormatRegistry::try_from(x).map(|a| a.into())
	}
}

impl<'a> TryFrom<&'a str> for Ss58AddressFormatRegistry {
	type Error = ParseError;

	fn try_from(x: &'a str) -> Result<Ss58AddressFormatRegistry, Self::Error> {
		ALL_SS58_ADDRESS_FORMAT_NAMES
			.binary_search(&x)
			.map(|lookup| ALL_SS58_ADDRESS_FORMATS[lookup])
			.map_err(|_| ParseError)
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

#[cfg(feature = "std")]
impl std::str::FromStr for Ss58AddressFormatRegistry {
	type Err = ParseError;

	fn from_str(data: &str) -> Result<Self, Self::Err> {
		TryFrom::try_from(data)
	}
}

#[cfg(test)]
mod tests {
	use super::{Ss58AddressFormat, Ss58AddressFormatRegistry};
	use std::convert::TryInto;

	#[test]
	fn is_reserved() {
		let reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::Reserved46Account.into();
		assert!(reserved.is_reserved());

		let not_reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadexAccount.into();
		assert!(!not_reserved.is_reserved());

		assert!(!Ss58AddressFormat::custom(100).is_reserved());
	}

	#[test]
	fn is_custom() {
		assert!(Ss58AddressFormat::custom(432).is_custom());

		let reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::Reserved46Account.into();
		assert!(!reserved.is_custom());

		let not_reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadexAccount.into();
		assert!(!not_reserved.is_custom());
	}

	#[test]
	fn enum_to_name_and_back() {
		for name in Ss58AddressFormat::all_names() {
			let val: Ss58AddressFormatRegistry = (*name).try_into().expect(name);
			assert_eq!(name, &val.to_string());
		}
	}
}
