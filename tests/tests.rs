use sp_debug_derive::RuntimeDebug;
use ss58_registry::ss58_registry;

#[test]
fn test_can_construct() {
    ss58_registry!();
}
