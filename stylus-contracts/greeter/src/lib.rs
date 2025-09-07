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
    extern crate cdchunking;

    use std::fs;
    use std::path::PathBuf;
    // use cdchunking::{Chunker, ChunkInput, ZPAQ};
    // use cdc::*;
    // use cbor::{Decoder, Encoder};
    use base64::engine::{general_purpose, Engine as _};
    use std::io::{self, Cursor, Read, Write}; // Import flush function
    use std::io::BufReader;
    use filebuffer::FileBuffer;
    use md5::{Md5, Digest};
    use ciborium::{de::from_reader, ser::into_writer};
    use std::collections::VecDeque;
    use stylus_sdk::crypto::keccak;
    use stylus_sdk::testing::*;
    use super::*;

    /// Downloads the ONNX model file from Hugging Face if it doesn't exist locally
    fn download_test_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let url = "https://huggingface.co/onnx-community/SmolLM2-135M-Instruct-ONNX-MHA/resolve/main/onnx/model_quantized.onnx";
        let filename = "model.onnx";
        // let url = "https://storage.googleapis.com/nftimagebucket/tokens/0x0f6ecda8cf57ee1e6e41b8b7f73006dca5f7a426/preview/TVRjek1qazJPRGd5Tnc9PV82Mw==.gif";
        // let filename = "test.gif";
        let mut dest = std::env::temp_dir();
        dest.push(filename);
        println!("Downloading test file to: {:?}", dest);

        // Download the file only if it doesn't exist
        if !dest.exists() {
            let response = reqwest::blocking::get(url)
                .map_err(|e| format!("Failed to download file: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Download failed with status: {}", response.status()).into());
            }

            let bytes = response.bytes()
                .map_err(|e| format!("Failed to read response bytes: {}", e))?;
            
            fs::write(&dest, bytes)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }

        Ok(dest)
    }

    // #[inline]
    // fn separator_size_nb_bits() -> u64 {
    //     13
    // }

    #[inline]
    fn default_predicate(x: u64) -> bool {
        const BITMASK: u64 = (1u64 << 19) - 1;
        x & BITMASK == BITMASK
    }

    // #[test]
    // fn test_greeter() {
    //     let vm = TestVM::default();
    //     let mut contract = Greeter::from(&vm);

    //     contract.set_name("Alice".to_string());
    //     assert_eq!(contract.greet(), "Hello, Alice!");
    // }

    #[test]
    fn test_write_model_to_chain() {
        // the collection of model files keyed by their hashes
        let mut models: VecDeque<(String, Vec<u8>)> = VecDeque::new();

        // 1. Download the binary test file (a GIF).
        let model_path = download_test_file().expect("Must be a path").display().to_string();
        
        // 2. Open the file for reading.
        let model_buffer = FileBuffer::open(&model_path).expect("Failed to open model file");
        // assert_eq!(keccak(model_buffer.to_vec()).to_string(), "");

        // hash the model content
        let mut model_buffer_hasher = Md5::new();
        model_buffer_hasher.update(&model_buffer);

        // prepend model vector to the collection
        models.push_back((
            format!("{:x}", model_buffer_hasher.finalize()),
            model_buffer.to_vec(),
        ));

        // if let Some((model_hash, _)) = models.front() {
        //     assert_eq!(model_hash, "");
        // }

        // convert the model collection to cbor object
        let mut model_serialized = Vec::new();
        // let mut model_serialized = Vec::with_capacity(model_buffer.len() + 128);
        into_writer(&models, &mut model_serialized).unwrap();

        // deserialize the cbor object back to the model collection
        let mut decoded_models: VecDeque<(String, Vec<u8>)> = from_reader(&model_serialized[..]).unwrap();
        if let Some((model_hash_orig, _)) = models.front() {
            if let Some((model_hash, _)) = decoded_models.front() {
                assert_eq!(model_hash, model_hash_orig);
            }
        }

        // // 2. Open the file for reading.
        // let mut model_file = fs::File::open(model_path).unwrap();
        // let model_file_length = model_file.metadata().unwrap().len();
        // // assert!(model_file_length, 0);
        // let model_reader: BufReader<fs::File> = BufReader::new(model_file);
        // let model_byte_iter = model_reader.bytes().map(|b| b.unwrap());

        // // separator iterator for the model content
        // // let model_separator_iter = SeparatorIter::new(model_byte_iter);
        // let model_separator_iter = SeparatorIter::custom_new(model_byte_iter, 19, default_predicate);

        // // chunk iterator for the model content
        // let chunk_iter = ChunkIter::new(model_separator_iter, model_file_length);

        // let mut chunk_count = 0;

        // for chunk in chunk_iter {
        //     // assert_eq!(format!(
        //     //     "Index: {}, size: {:6}, separator_hash: {:016x}",
        //     //     chunk.index, chunk.size, chunk.separator_hash
        //     // ), "");
        //     chunk_count += 1;
        //     // Here you would typically write the chunk to the blockchain or process it further.
        // }

        // assert_eq!(chunk_count, 0);

        // // 3. Create a chunker.
        // let chunker = Chunker::new(ZPAQ::new(13));
        
        // // 4. Use `whole_chunks` to correctly process the binary data.
        // // The iterator returns a Result<Vec<u8>>, which must be handled.
        // let mut chunks: Vec<Vec<u8>> = Vec::new();
        // for chunk_result in chunker.whole_chunks(model_reader) {
        //     let chunk = chunk_result.expect("Error reading chunk from file");
        //     chunks.push(chunk);
        // }
        
        // // 5. Add your remaining test logic here.
        // // This is where you would normally assert the outcome or perform further actions.
        // assert!(!chunks.is_empty(), "Chunks should not be empty");
        // println!("Successfully processed {} binary chunks in the test.", chunks.len());

        // let vm = TestVM::default();
        // let mut contract = Greeter::from(&vm);
        
        assert_eq!("Done!", "Done!");
    }
}
