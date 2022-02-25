/// Error encountered while parsing `Ss58AddressFormat` from &'_ str
/// unit struct for now.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ParseError;

#[cfg(feature = "std")]
impl std::fmt::Display for ParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "failed to parse network value as u16")
	}
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
