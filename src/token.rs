// Copyright (C) 2021-2022 Parity Technologies (UK) Ltd.
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

#[cfg(feature = "std")]
use num_format::{CustomFormat, ToFormattedString};

/// A given amount of token. Can be used for nicely formatted output and token-aware comparison of
/// different amounts.
///
/// ```
/// # use ss58_registry::Token;
/// # #[cfg(feature = "std")]
/// # fn x() {
/// let my_token = Token { name: "I❤U", decimals: 8, amount: 100_000_000_000 };
/// assert_eq!(format!("{}", my_token), "1_000,000 I❤U");
/// assert_eq!(format!("{:?}", my_token), "1000,000 I❤U (100_000_000_000)");
/// # }
/// # #[cfg(not(feature = "std"))]
/// # fn x() {}
/// # x();
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
	/// The short name (ticker) of the token
	pub name: &'static str,
	/// The number of decimals the token has (smallest granularity of the token)
	pub decimals: u8,
	/// The amount in the smallest granularity of the token.
	pub amount: u128,
}

#[cfg(feature = "std")]
impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let multiplier = u128::pow(10, self.decimals as u32);
		let format = CustomFormat::builder().decimal(",").separator("_").build().unwrap();
		write!(
			f,
			"{},{:0>3} {}",
			(self.amount / multiplier).to_formatted_string(&format),
			self.amount % multiplier / (multiplier / 1000),
			self.name
		)
	}
}

#[cfg(feature = "std")]
impl std::fmt::Debug for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let multiplier = u128::pow(10, self.decimals as u32);
		let format = CustomFormat::builder().decimal(",").separator("_").build().unwrap();
		write!(
			f,
			"{},{:0>3} {} ({})",
			self.amount / multiplier,
			self.amount % multiplier / (multiplier / 1000),
			self.name,
			self.amount.to_formatted_string(&format),
		)
	}
}
