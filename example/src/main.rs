#![cfg_attr(not(feature="std"),feature(lang_items))]
#![cfg_attr(not(feature="std"),no_std)]
#![warn(missing_docs)]
//! Example of macro with 'std' feature enabled by default.
//! and check it builds as no_std when --no-default-features.
use sp_debug_derive::RuntimeDebug;
use ss58_registry::ss58_registry;

ss58_registry!();

use core::panic::PanicInfo;

#[cfg(not(feature="std"))]
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(not(feature="std"))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Example
pub fn main() {
    assert!(Ss58AddressFormat::Custom(1) != Ss58AddressFormat::Custom(2));
}
