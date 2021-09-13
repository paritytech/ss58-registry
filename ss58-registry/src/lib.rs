#![warn(missing_docs)]

//! Example of macro with 'std' feature enabled by default.
//! and check it builds as no_std when --no-default-features.
use sp_debug_derive::RuntimeDebug;
use ss58_registry_derive::ss58_registry_derive;

ss58_registry_derive!();
