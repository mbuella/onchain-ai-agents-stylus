//!
//! Stylus Hello World
//!
//! The following contract implements a simple Greeter contract.
//!
//! ```solidity
//! contract Greeter {
//!     string public name;
//!     function setName(string memory newName) public {
//!         name = newName;
//!     }
//!     function greet() public view returns (string memory) {
//!         return string(abi.encodePacked("Hello, ", name, "!"));
//!     }
//! }
//! ```
//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;
use stylus_sdk::storage::*;

// Define some persistent storage using the Solidity ABI.
// `Greeter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Greeter {
        StorageString name;
    }
}

/// Declare that `Greeter` is a contract with the following external methods.
#[public]
impl Greeter {
    /// Gets the name from storage.
    pub fn name(&self) -> String {
        self.name.get_string()
    }

    /// Sets the name in storage to a user-specified value.
    pub fn set_name(&mut self, new_name: String) {
        self.name.set_str(&new_name);
    }

    /// Returns a greeting message using the stored name.
    pub fn greet(&self) -> String {
        let name = self.name.get_string();
        format!("Hello, {}!", name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    #[test]
    fn test_greeter() {
        let vm = TestVM::default();
        let mut contract = Greeter::from(&vm);

        contract.set_name("Alice".to_string());
        assert_eq!(contract.greet(), "Hello, Alice!");
    }
}
