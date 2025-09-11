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

use ciborium::{de::from_reader_with_buffer, de::from_reader, ser::into_writer};
use borsh::{BorshSerialize, BorshDeserialize};
// use std::collections::VecDeque;
use md5::{Md5, Digest};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::alloy_primitives::{address, Address, fixed_bytes, FixedBytes, U256, B256};
use stylus_sdk::abi::Bytes;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::*;
use stylus_sdk::crypto::keccak;

// // Define some persistent storage using the Solidity ABI.
// // `Greeter` will be the entrypoint.
// sol_storage! {
//     #[entrypoint]
//     pub struct Greeter {
//         StorageMap<FixedBytes<32>, StorageBytes> agent_revisions;
//         StorageBytes agent_revisions_serialized;
//         // StorageMap<FixedBytes<4>,StorageBytes> agent_revisions_serialized;
//         StorageBytes agent_model;
//         StorageString name;
//     }
// }
#[storage]
#[entrypoint]
pub struct Greeter {
    // agent_revisions: StorageMap<FixedBytes<32>, StorageBytes>,
    agent_serialized: StorageMap<U256, StorageBytes>,
    agent_serialized_manifests: StorageMap<FixedBytes<32>, StorageVec<StorageU256>>,
    agent_serialized_latest_manifest_hash: StorageFixedBytes<32>,
    name: StorageString,
}

/// Declare that `Greeter` is a contract with the following external methods.
#[public]
impl Greeter {
    pub fn get_agent_latest_revision(&mut self) -> Vec<u8> {
        // get the latest revision manifest
        let agent_serialized_manifest_parts = self.agent_serialized_manifests
            .get(self.agent_serialized_latest_manifest_hash.get());

        // agent_serialized_manifest_parts

        // build the revision based on the manifest parts
        (0..agent_serialized_manifest_parts.len())
            .map(|i| agent_serialized_manifest_parts.get(i).unwrap())
            .flat_map(|chunk_hash| {
                self.agent_serialized.get(chunk_hash).get_bytes().into_iter()
            })
            .collect()
    }

    pub fn add_agent_revision(&mut self, agent: Vec<u8>) {
        let mut agent_hasher = Md5::new();
        agent_hasher.update(&agent);
        let agent_hash = format!("{:x}", agent_hasher.finalize());

        // break down the agent content (bytes) into chunks
        // // let chunk_avg_size: u32 = 131072; // 128kb
        // // let chunk_avg_size: u32 = 262144; // 256kb
        let chunk_avg_size: u32 = 1024 * 1024 * 3; // 3mb
        let chunk_min_size = chunk_avg_size / 4;
        let chunk_max_size = chunk_avg_size * 4;
        // let chunk_max_size: u32 = 1024 * 1024 * 2;

        let agent_chunks = fastcdc::v2020::FastCDC::with_level(
            &agent,
            // fastcdc::v2020::MINIMUM_MIN,
            // chunk_max_size / 2,
            // chunk_max_size,
            chunk_min_size,
            chunk_avg_size,
            chunk_max_size,
            fastcdc::v2020::Normalization::Level3,
        );
        
        // now start the chunking and building the manifest
        let mut existing_chunk_count = 0;
        let agent_revision_manifest: Vec<U256> = agent_chunks.map(|chunk| {
            let chunk_hash = U256::from(chunk.hash);

            if self.agent_serialized.get(chunk_hash).is_empty() {
                self.agent_serialized
                    .setter(chunk_hash)
                    .set_bytes(&agent[chunk.offset..(chunk.offset + chunk.length)]);
            } else {
                existing_chunk_count += 1;
            }

            chunk_hash
        }).collect();

        // assert_eq!(existing_chunk_count, 0);

        // u8 * 32 = U256 😅
        let agent_revision_manifest_hash = keccak(
            agent_revision_manifest
                .iter()
                .flat_map(|hash| hash.to_be_bytes::<32>())
                .collect::<Vec<u8>>()
        ).into();

        // loop thru each manifest part to build the manifest
        agent_revision_manifest.iter().for_each(|part|
            self.agent_serialized_manifests
                .setter(agent_revision_manifest_hash)
                .push(*part)
        );

        // mark the manifest as latest
        self.agent_serialized_latest_manifest_hash.set(agent_revision_manifest_hash);
    }

    // pub fn add_agent_revision_direct_storagemap(&mut self, agent: Vec<u8>) {
    //     let mut agent_hasher = Md5::new();
    //     agent_hasher.update(&agent);
    //     let agent_hash = format!("{:x}", agent_hasher.finalize());

    //     self.agent_revisions
    //         .setter(keccak(agent_hash.as_bytes().to_vec()).into())
    //         .set_bytes(agent);
    // }

    // pub fn add_agent_revision_old(&mut self, agent: Vec<u8>) {
    //     let mut agent_hasher = Md5::new();
    //     agent_hasher.update(&agent);
    //     let agent_hash = format!("{:x}", agent_hasher.finalize());

    //     let mut agent_revisions: Vec<(String, Vec<u8>)> = Vec::new();
    //     if self.agent_revisions_serialized.len() > 0 {
    //         // ciborium
    //         // let mut decoded_agent_scratch = vec![0u8; 2048];
    //         // agent_revisions = from_reader_with_buffer(
    //         //     &self.agent_revisions_serialized.get_bytes()[..],
    //         //     &mut decoded_agent_scratch[..]
    //         // ).unwrap();
    //         // agent_revisions = from_reader(&self.agent_revisions_serialized.get_bytes()[..]).unwrap();

    //         // borsh
    //         // agent_revisions = borsh::from_slice(&self.agent_revisions_serialized.get_bytes()).unwrap();
    //         agent_revisions = borsh::from_slice(&self.agent_revisions_serialized.load()).unwrap();
    //     }

    //     agent_revisions.insert(0, (agent_hash, agent));

    //     /// ciborium
    //     // let mut agent_revisions_serialized = Vec::new();
    //     // into_writer(&agent_revisions, &mut agent_revisions_serialized).unwrap();

    //     // borsh
    //     let mut agent_revisions_serialized = borsh::to_vec(&agent_revisions).unwrap();
    //     // if agent_revisions.len() > 1 {
    //     //     agent_revisions_serialized.dedup();
    //     // }

    //     self.agent_revisions_serialized.set_bytes(&agent_revisions_serialized);
    // }

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
    use ciborium::{de::from_reader, de::from_reader_with_buffer, ser::into_writer};
    use std::collections::VecDeque;
    use stylus_sdk::storage::*;
    use stylus_sdk::crypto::keccak;
    use stylus_sdk::testing::*;
    use super::*;

    // model download helper
    fn download_model(url: &str, filename: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
        // this is the actual model, which we store as a list of its revisions
        // we key each revision by its md5 hash
        let mut model: VecDeque<(String, Vec<u8>)> = VecDeque::new();

        // our test arb stylus vm
        let vm = TestVM::default();
        let mut contract = Greeter::from(&vm);

        // get the first revision of the model
        let model_rev1_path = download_model(
            "https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/main/onnx/model_quantized.onnx",
            "model_rev1.onnx"
        ).expect("Must be a path").display().to_string();
        let model_rev1_buffer = FileBuffer::open(&model_rev1_path).expect("Failed to open model file");
        // assert_eq!(keccak(model_buffer.to_vec()).to_string(), "");

        // let model_rev1_save_start_time = std::time::Instant::now();
        contract.add_agent_revision(model_rev1_buffer.to_vec());
        // let model_rev1_save_elapsed_time = model_rev1_save_start_time.elapsed(); // End timer
        // assert_eq!(format!("Time taken for saving model rev1: {:?}", model_rev1_save_elapsed_time), "");

        // // get the latest model revision
        // let latest_agent_revision = contract.get_agent_latest_revision();
        // // the latest revision should be equal to the recently added revision
        // assert_eq!(latest_agent_revision.len(), model_rev1_buffer.to_vec().len());

        // // hash the model content
        // let mut model_rev1_hasher = Md5::new();
        // model_rev1_hasher.update(&model_rev1_buffer);

        // // prepend model vector to the collection
        // model.push_front((
        //     format!("{:x}", model_rev1_hasher.finalize()),
        //     model_rev1_buffer.to_vec(),
        // ));

        // // if let Some((model_hash, _)) = models.front() {
        // //     assert_eq!(model_hash, "");
        // // }

        // // convert the model collection to cbor object
        // let mut model_serialized = Vec::new();
        // // let mut model_serialized = Vec::with_capacity(model_buffer.len() + 128);
        // into_writer(&model, &mut model_serialized).unwrap();

        // // save the serialized model as field in test smart contract
        // // let start_time = std::time::Instant::now();
        // contract.set_agent_model(model_serialized);
        // // let elapsed_time = start_time.elapsed(); // End timer
        // // assert_eq!(format!("Time taken for saving model rev1: {:?}", elapsed_time), "");

        // get the second revision of the model
        let model_rev2_path = download_model(
            "https://huggingface.co/onnx-community/SmolLM2-135M-Instruct-ONNX-MHA/resolve/main/onnx/model_quantized.onnx",
            "model_rev2.onnx"
        ).expect("Must be a path").display().to_string();
        let model_rev2_buffer = FileBuffer::open(&model_rev2_path).expect("Failed to open model file");
        // // assert_eq!(keccak(model_buffer.to_vec()).to_string(), "");

        // let model_rev2_save_start_time = std::time::Instant::now();
        contract.add_agent_revision(model_rev2_buffer.to_vec());
        // let model_rev2_save_elapsed_time = model_rev2_save_start_time.elapsed(); // End timer
        // assert_eq!(format!("Time taken for saving model rev2: {:?}", model_rev2_save_elapsed_time), "");

        // get the latest model revision
        // let get_latest_model_start_time = std::time::Instant::now();
        let latest_agent_revision = contract.get_agent_latest_revision();
        // let get_latest_model_elapsed_time = get_latest_model_start_time.elapsed(); // End timer
        assert_eq!(latest_agent_revision, model_rev2_buffer.to_vec(), "Not the same!");
        // assert_eq!(format!("Time taken for loading the latest model: {:?}", get_latest_model_elapsed_time), "");

        // let mut latest_agent_revision_hasher = Md5::new();
        // latest_agent_revision_hasher.update(&latest_agent_revision);
        // let mut model_rev2_buffer_hasher = Md5::new();
        // model_rev2_buffer_hasher.update(&model_rev2_buffer.to_vec());

        // // the latest revision should be equal to the recently added revision
        // assert_eq!(
        //     format!("{:x}", latest_agent_revision_hasher.finalize()),
        //     format!("{:x}", model_rev2_buffer_hasher.finalize())
        // );

        // // hash the model content
        // let mut model_rev2_hasher = Md5::new();
        // model_rev2_hasher.update(&model_rev2_buffer);

        // // prepend model vector to the collection
        // model.push_front((
        //     format!("{:x}", model_rev2_hasher.finalize()),
        //     model_rev2_buffer.to_vec(),
        // ));

        // // convert the model collection to cbor object
        // // let mut model_serialized_v2 = Vec::new();
        // into_writer(&model, &mut model_serialized).unwrap();

        // // save the serialized model as field in test smart contract
        // let start_time = std::time::Instant::now();
        // contract.set_agent_model(model_serialized);
        // let elapsed_time = start_time.elapsed(); // End timer
        // assert_eq!(format!("Time taken for saving model rev2: {:?}", elapsed_time), "");
        




        // // deserialize the cbor object back to the model collection
        // let start_time = std::time::Instant::now();
        // // let mut decoded_models: VecDeque<(String, Vec<u8>)> = from_reader(&model_serialized[..]).unwrap();
        // // let mut decoded_models_scratch_buffer = vec![0u8; 1024 * 3072]; // 2mb buffer
        // let mut decoded_models_scratch_buffer = vec![0u8; 2048];
        // let mut decoded_models: VecDeque<(String, Vec<u8>)> = from_reader_with_buffer(&model_serialized[..], &mut decoded_models_scratch_buffer[..]).unwrap();
        // if let Some((model_hash_orig, _)) = models.front() {
        //     if let Some((model_hash, _)) = decoded_models.front() {
        //         assert_eq!(model_hash, model_hash_orig);
        //     }
        // }
        // let elapsed_time = start_time.elapsed(); // End timer
        // assert_eq!(format!("Time taken for model processing: {:?}", elapsed_time), "");

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

        assert_eq!("Done!", "Done!");
    }
}
