use super::*;

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

	/// Address prefix used on the network
	pub const fn prefix(&self) -> u16 {
		self.prefix
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

/// const function to convert [`Ss58AddressFormat`] to u16
pub const fn from_address_format(x: Ss58AddressFormat) -> u16 {
	x.prefix
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
