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

use alloc::vec::Vec;
use alloc::string::{String, ToString};
// use alloc::collections::{VecDeque, BTreeMap};
use alloc::collections::{BTreeMap, BTreeSet};

// use hashbrown::HashMap;
// use vector_map::VecMap as Map;

// extern crate fxhash;
// use fxhash::FxHashMap;

// use seq_chunking::{SeqChunking, ChunkingConfig, SeqOpMode};
use hash_roll::zstd::Zstd;
use hash_roll::{ToChunkIncr, ChunkIncr};
// use hash_roll::fastcdc::FastCdc;
// use hash_roll::{ToChunkIncr, ChunkIncr, gear_table::GEAR_64};

// use ciborium::{de::from_reader_with_buffer, de::from_reader, ser::into_writer};
// use ruzstd::encoding::{compress, compress_to_vec, FrameCompressor,  CompressionLevel};
// use ruzstd::decoding::FrameDecoder;
// use miniz_oxide::deflate::compress_to_vec;
// use miniz_oxide::inflate::decompress_to_vec_with_limit;
// use core2::io::{Read, Write};
// use libflate::gzip::{Encoder, Decoder};
// use borsh::{BorshSerialize, BorshDeserialize};
// use bitcode::{Encode, Decode};
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};
use rkyv::{api::high::to_bytes_with_alloc, ser::allocator::Arena};
// use std::collections::VecDeque;
use md5::{Md5, Digest};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::alloy_primitives::{FixedBytes};
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

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub struct AgentRevisionManifests(pub BTreeMap<String, Vec<u64>>);
// #[derive(Encode, Decode, Debug)]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct AgentRevision {
    pub revisions: BTreeMap<[u8; 16], Vec<u8>>,
    pub manifests: BTreeMap<[u8; 16], Vec<[u8; 16]>>,
    pub latest_hash: [u8; 16],
}

#[storage]
#[entrypoint]
pub struct Greeter {
    // agent_revisions: StorageMap<FixedBytes<32>, StorageBytes>,
    // agent_serialized: StorageMap<FixedBytes<8>, StorageBytes>,
    agent_serialized: StorageMap<FixedBytes<16>, StorageBytes>,
    // agent_serialized_manifests: StorageMap<FixedBytes<32>, StorageVec<StorageU256>>,
    agent_serialized_manifests_serialized: StorageBytes,
    // agent_serialized_latest_manifest_hash: StorageFixedBytes<32>,
    // agent_serialized_latest_manifest_hash: StorageBytes,
    name: StorageString,
}

/// Declare that `Greeter` is a contract with the following external methods.
#[public]
impl Greeter {
    pub fn get_agent_latest_revision(&mut self) -> Vec<u8> {
        // // return empty vec if agent_serialized_manifests_serialized is empty
        // if self.agent_serialized_manifests_serialized.len() == 0 {
        //     Vec::new()
        // }

        // get the latest revision manifest
        // agent_revision = bitcode::decode(&agent_serialized_manifests_serialized).unwrap();
        // let agent_revision_manifests: BTreeMap<String, Vec<u64>> = borsh::from_slice(&self.agent_serialized_manifests_serialized.get_bytes()).unwrap();
        let agent_serialized_manifests_serialized: Vec<u8> = self.agent_serialized_manifests_serialized.get_bytes();
        let agent_revision_archived = rkyv::access::<ArchivedAgentRevision, Error>(&agent_serialized_manifests_serialized[..]).unwrap();
        let agent_revision = deserialize::<AgentRevision, Error>(agent_revision_archived).unwrap();
        // assert_eq!(
        //     agent_revision_manifests.get(&self.agent_serialized_latest_manifest_hash.get().to_string()).unwrap().len(),
        //     0 
        // );

    //     // get the latest agent revision
    //     // Create a HashSet for fast lookups
    //     let key_set: BTreeSet<> = keys_to_keep.into_iter().collect();
    
    // // Create a Vec<u8> from concatenated Vec<u8>s of filtered values
    // let concatenated_vec: Vec<u8> = map
    //     .iter()
    //     .filter(|(key, _)| key_set.contains(key))
    //     .flat_map(|(_, value)| value.clone()) // Flatten the Vec<u8>s
    //     .collect();

        // let agent_revision_manifest = agent_revision_manifests.get(
        //     &self.agent_serialized_latest_manifest_hash.get().to_string()
        // ).unwrap();
        // let agent_revision = 
        agent_revision.manifests
            // get the latest agent revision manifest
            .get(&agent_revision.latest_hash)
            .unwrap()
            // now rebuild the revision bytes
            .iter()
            // .flat_map(|part| {
            //     let chunk_key = FixedBytes::<16>::new(*part);
    
            //     self.agent_serialized.get(chunk_key).get_bytes().into_iter()
            // })
            .flat_map(|chunk_hash_bytes| {
                // assert_eq!(*agent_revision.revisions.get(chunk_hash_bytes).unwrap(), 0);
                // agent_revision.revisions.get(chunk_hash_bytes).unwrap().clone()
                let chunk_hash: FixedBytes<16> = FixedBytes::from_slice(chunk_hash_bytes);
    
                self.agent_serialized.get(chunk_hash).get_bytes().into_iter()
            })
            .collect::<Vec<_>>()
            // .concat()
            // .cloned()

        // // build the revision based on the manifest parts
        // agent_revision_manifest.iter()
        //     .flat_map(|part| {
        //         let chunk_hash: FixedBytes<8> = FixedBytes::from_slice(&part.to_be_bytes());
    
        //         self.agent_serialized.get(chunk_hash).get_bytes().into_iter()
        //     })
        //     .collect()

        // let agent_serialized_manifest_parts = self.agent_serialized_manifests
        //     .get(self.agent_serialized_latest_manifest_hash.get());

        // // agent_serialized_manifest_parts

        // // build the revision based on the manifest parts
        // (0..agent_serialized_manifest_parts.len())
        //     .map(|i| agent_serialized_manifest_parts.getter(i).as_deref().unwrap().get())
        //     .flat_map(|chunk_hash| {
        //         self.agent_serialized.get(chunk_hash).get_bytes().into_iter()
        //     })
        //     .collect()
    }
 
    pub fn add_agent_revision(&mut self, agent: Vec<u8>) {
        // let mut agent_hasher = Md5::new();
        // agent_hasher.update(&agent);
        // let agent_hash = format!("{:x}", agent_hasher.finalize());

        // let mut agent_revision_manifests = Map::new();
        // let mut agent_revision_manifests: Vec<(String,Vec<u64>)> = Vec::new();
        // let mut agent_revision_manifests: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        let mut agent_revision = AgentRevision {
            revisions: BTreeMap::new(),
            manifests: BTreeMap::new(),
            latest_hash: [0; 16],
        };
        // load agent_revision_manifests from the blockchain if it exists
        if self.agent_serialized_manifests_serialized.len() > 0 {
            // let agent_serialized_manifests_serialized: Vec<u8> = (0..agent_serialized_manifests_serialized_len)
            //     .map(|i| self.agent_serialized_manifests_serialized.get(i).unwrap())
            //     .collect();
            let agent_serialized_manifests_serialized: Vec<u8> = self.agent_serialized_manifests_serialized.get_bytes();

            // agent_revision = bitcode::decode(&agent_serialized_manifests_serialized).unwrap();
            let agent_revision_archived = rkyv::access::<ArchivedAgentRevision, Error>(&agent_serialized_manifests_serialized[..]).unwrap();
            agent_revision = deserialize::<AgentRevision, Error>(agent_revision_archived).unwrap();
        }

        // assert_eq!(agent_revision.latest_hash, [0; 16]);

        // // break down the agent content (bytes) into chunks
        // // // let chunk_avg_size: u32 = 131072; // 128kb
        // // // let chunk_avg_size: u32 = 262144; // 256kb
        // let chunk_avg_size: u64 = 65536; // 64kb
        let chunk_avg_size: u64 = 131072; // 128kb
        // let chunk_avg_size: u64 = 262144; // 256kb
        // let chunk_avg_size: u64 = 1024 * 1024 * 3; // 3mb
        let chunk_min_size = chunk_avg_size / 4;
        let chunk_max_size = chunk_avg_size * 4;
        // // let chunk_max_size: u32 = 1024 * 1024 * 2;

        let mut agent_revision_chunkhashes_bytes: Vec<u8> = Vec::new();
        let mut agent_revision_manifest_hasher = Md5::new();
        let mut agent_revision_manifest: Vec<[u8; 16]> = Vec::new();

        let zstd = Zstd::with_target_section_size(1024 * 1024 * 2);
        // let zstd = FastCdc::default();
        // let zstd = FastCdc::new(
        //     &GEAR_64,
        //     chunk_min_size,
        //     chunk_avg_size,
        //     chunk_max_size,
        // );
        let mut incr = zstd.to_chunk_incr();
        let mut zstd_cursor = 0;
        let slice = &agent[zstd_cursor..];

        let mut ctr: i32 = 0;

        // #[inline]
        // fn insert_chunk(mut agent_revisions: BTreeMap<[u8; 16], Vec<u8>>, mut agent_revision_manifest: Vec<[u8; 16]>, mut agent_revision_manifest_hasher: impl Digest, chunk: &[u8], ctr: &mut i32) {
        //     let chunk_hash = Md5::digest(&chunk);
        //     let chunk_hash_bytes: [u8; 16] = chunk_hash.into();
            
        //     // insert unique chunks to the revisions map
        //     if !agent_revisions.contains_key(&chunk_hash_bytes) {
        //         agent_revisions.insert(
        //             chunk_hash_bytes,
        //             chunk.to_vec(),
        //         );
        //     }

        //     // add the current chunk hash to rev manifest map
        //     agent_revision_manifest.push(chunk_hash_bytes);

        //     // update the current revision hasher with the current chunk
        //     agent_revision_manifest_hasher.update(chunk_hash_bytes.as_slice());

        //     *ctr += 1;
        // }

        while zstd_cursor < agent.len() {
            let slice = &agent[zstd_cursor..];
            if let Some(chunk_len) = incr.push(slice) {
                // insert_chunk(agent_revision.revisions, agent_revision_manifest, agent_revision_manifest_hasher, &slice[..chunk_len], &mut ctr);
                let chunk = &slice[..chunk_len];
                let chunk_hash = Md5::digest(&slice[..chunk_len]);
                let chunk_hash_bytes: [u8; 16] = chunk_hash.into();
                let chunk_key = FixedBytes::<16>::from(&chunk_hash_bytes);
                
                // insert unique chunks to the revisions map
                // if !agent_revision.revisions.contains_key(&chunk_hash_bytes) {
                //     agent_revision.revisions.insert(
                //         chunk_hash_bytes,
                //         chunk.to_vec(),
                //     );

                //     ctr += 1;
                // }
                if self.agent_serialized.get(chunk_key).is_empty() {
                    self.agent_serialized
                        .setter(chunk_key)
                        .set_bytes(&chunk);
                }

                // add the current chunk hash to rev manifest map
                agent_revision_manifest.push(chunk_hash_bytes);

                // update the current revision hasher with the current chunk
                agent_revision_manifest_hasher.update(chunk_hash_bytes.as_slice());

                zstd_cursor += chunk_len;
            } else {
                // insert_chunk(agent_revision.revisions, agent_revision_manifest, agent_revision_manifest_hasher, &slice, &mut ctr);
                let chunk = &slice;
                let chunk_hash = Md5::digest(&chunk);
                let chunk_hash_bytes: [u8; 16] = chunk_hash.into();
                let chunk_key = FixedBytes::<16>::from(&chunk_hash_bytes);
                
                // insert unique chunks to the revisions map
                // if !agent_revision.revisions.contains_key(&chunk_hash_bytes) {
                //     agent_revision.revisions.insert(
                //         chunk_hash_bytes,
                //         chunk.to_vec(),
                //     );

                //     ctr += 1;
                // }
                if self.agent_serialized.get(chunk_key).is_empty() {
                    self.agent_serialized
                        .setter(chunk_key)
                        .set_bytes(&chunk);
                }

                // add the current chunk hash to rev manifest map
                agent_revision_manifest.push(chunk_hash_bytes);

                // update the current revision hasher with the current chunk
                agent_revision_manifest_hasher.update(chunk_hash_bytes.as_slice());


                break;
            }
        }

        // assert_eq!(ctr, 78);

        // // Create a chunker with default settings
        // let chunker = SeqChunking::new();
        // // let chunker_config = ChunkingConfig::builder()
        // //     // .seq_threshold(4)                    // Longer sequences needed
        // //     // .op_mode(SeqOpMode::Decreasing)       // Look for decreasing sequences
        // //     // .jump_trigger(100)                    // Jump after 100 opposing slopes
        // //     // .min_block_size(chunk_min_size)
        // //     // .max_block_size(chunk_max_size)
        // //     .avg_block_size(chunk_avg_size)
        // //     .build()
        // //     .expect("Invalid configuration");
        // // let chunker = SeqChunking::from_config(chunker_config);
        // // let agent_chunks: Vec<_> = chunker.chunk_all(&agent).collect();

        // // // let agent_chunks = fastcdc::v2020::FastCDC::with_level(
        // // //     &agent,
        // // //     // fastcdc::v2020::MINIMUM_MIN,
        // // //     // chunk_max_size / 2,
        // // //     // chunk_max_size,
        // // //     chunk_min_size,
        // // //     chunk_avg_size,
        // // //     chunk_max_size,
        // // //     fastcdc::v2020::Normalization::Level3,
        // // // );
        
        // // let mut agent_revision_manifest: Vec<u8> = Vec::new();
        // // let mut agent_revision_chunkhashes_bytes: Vec<u8> = Vec::new();
        // // let mut agent_revision_manifest_hasher = Md5::new();
        // // let agent_revision_manifest: Vec<u64> = Vec::new();
        // // let mut agent_revision_manifest: Vec<u64> = agent_chunks.clone().into_iter().map(|chunk| {
        // // let agent_revision_manifest: Vec<u64> = agent_chunks.clone().into_iter().map(|chunk| {
        // let mut ctr = 0;
        // let agent_revision_manifest: Vec<[u8; 16]> = chunker.chunk_all(&agent).collect::<Vec<_>>().into_iter()
        //     .map(|chunk| {
        //         let chunk_hash = Md5::digest(&chunk.data);
        //         let chunk_hash_bytes: [u8; 16] = chunk_hash.into();
        //         let chunk_key = FixedBytes::<16>::from(&chunk_hash_bytes);
                                
        //         // save the chunk if it not already exists
        //         // if !agent_revision.revisions.contains_key(&chunk_hash_bytes) {
        //         //     agent_revision.revisions.insert(
        //         //         chunk_hash_bytes,
        //         //         chunk.data.to_vec(),
        //         //     );
                    
        //         //     ctr += 1;
        //         // }
        //         if self.agent_serialized.get(chunk_key).is_empty() {
        //             self.agent_serialized
        //                 .setter(chunk_key)
        //                 .set_bytes(&chunk.data);
        //         }

        //         // update the rev manifest with the current chunk hash
        //         agent_revision_manifest_hasher.update(chunk_hash_bytes.as_slice());

        //         // Return the GenericArray for flattening.
        //         // chunk_hash
        //         chunk_hash_bytes
        //     })
        //     // // generate hash for each chunk
        //     // .flat_map(|chunk_hash| chunk_hash.into_iter())
        //     .collect();

        // // assert_eq!(17500, ctr); // old, new

        // add the agent revision manifest
        let agent_revision_hash: [u8; 16] = agent_revision_manifest_hasher.finalize().into();
        agent_revision.manifests.insert(
            agent_revision_hash.clone(),
            agent_revision_manifest.clone()
        );

        // mark the agent revision as latest
        agent_revision.latest_hash = agent_revision_hash;

        // // serialize agent revision manifests via bitcode
        // let agent_revision_manifests_serialized = bitcode::encode(&agent_revision);
        // serialize agent revision manifests via rkyv
        // let agent_revision_manifests_serialized = rkyv::to_bytes::<Error>(&agent_revision).unwrap();
        let mut arena = Arena::new();
        let agent_revision_manifests_serialized = to_bytes_with_alloc::<_, Error>(&agent_revision, arena.acquire()).unwrap();

        // save agent_revision_manifests_serialized onchain
        self.agent_serialized_manifests_serialized.set_bytes(agent_revision_manifests_serialized);
        // assert_eq!(self.agent_serialized_manifests_serialized.len(), 0);

            // .collect::<Vec<_>>().into_iter()
            // // append each chunk hash to agent_revision_chunkhashes_bytes
            // .map(|(chunk_data, &chunk_hash)| {
            //     assert_eq!(chunk_hash.len(), 0);
            //     // agent_revision_chunkhashes_bytes.extend_from_slice(chunk_hash);
                
            //     // chunk_hash
            // });
            // .collect();

            // assert_eq!(agent_revision_manifest.len(), 0);

        // let agent_revision_manifest: Vec<u64> = chunker.chunk_all(&agent).into_iter().map(|chunk| {
        //     let mut chunk_hasher = Md5::new();
        //     chunk_hasher.update(&chunk.data);

        //     // add chunks to agent_serialized in chain
        //     // let chunk_hash_binding = keccak(chunk.data);
        //     let chunk_hash_binding = chunk_hasher.finalize();
        //     let chunk_hash_raw = chunk_hash_binding.as_slice();
        //     let chunk_hash_bytes = u64::from_be_bytes(chunk_hash_raw[0..8].try_into().unwrap());
        //     let chunk_hash: FixedBytes<8> = FixedBytes::from_slice(&chunk_hash_bytes.to_be_bytes());
        //     // let chunk_hash: FixedBytes<8> = FixedBytes::from_slice(&chunk.hash.to_be_bytes());

        //     if self.agent_serialized.get(chunk_hash).is_empty() {
        //         self.agent_serialized
        //             .setter(chunk_hash)
        //             // .set_bytes(&agent[chunk.offset..(chunk.offset + chunk.length)]);
        //             .set_bytes(&chunk.data);
        //     }
            
        //     // append chunk hash to agent_revision_chunkhashes_bytes
        //     // agent_revision_chunkhashes_bytes.extend_from_slice(&chunk.hash.to_le_bytes());
        //     agent_revision_chunkhashes_bytes.extend_from_slice(&chunk_hash_bytes.to_le_bytes());

        //     chunk_hash_bytes
        //     // chunk.hash
        // }).collect();

        // // Calculate the manifest hash from the collected chunk hash bytes.
        // let mut agent_revision_manifest_hasher = Md5::new();
        // agent_revision_manifest_hasher.update(&agent_revision_chunkhashes_bytes);
        // // let agent_revision_manifest_hash = agent_revision_manifest_hasher.finalize().to_vec();
        // let agent_revision_manifest_hash = format!("{:x}", agent_revision_manifest_hasher.finalize());
        // // let agent_revision_manifest_hash = keccak(agent_revision_chunkhashes_bytes);

        // // agent_revision_manifests.insert(
        // //     agent_revision_manifest_hash.to_string(),
        // //     agent_revision_manifest.clone()
        // // );
        // // agent_revision_manifests.push((
        // //     agent_revision_manifest_hash.to_string(),
        // //     agent_revision_manifest.clone()
        // // ));
        // // agent_revision_manifests.insert(
        // //     agent_revision_manifest_hash.to_string(),
        // //     agent_revision_manifest.clone()
        // // );
        // agent_revision.manifests.insert(
        //     agent_revision_manifest_hash.clone(),
        //     agent_revision_manifest.clone()
        // );
        // agent_revision.latest_manifest_hash = agent_revision_manifest_hash.clone();
        // // assert_eq!(
        // //     agent_revision_manifests.get("0xa6c042c7b58b39472428c3adea61e452ac68bf76af8fbff37bb1f5141b24e7c9").unwrap(),
        // //     &agent_revision_manifest,
        // //     "It works!"
        // // );
        // // assert_ne!(
        // //     agent_revision_manifests.iter().filter(|&(hash, manifest)| hash == "0xa6c042c7b58b39472428c3adea61e452ac68bf76af8fbff37bb1f5141b24e7c9").next().unwrap().1,
        // //     agent_revision_manifest,
        // //     "It works!"
        // // );
        // // assert_ne!(
        // //     agent_revision_manifests.into_keys().collect::<Vec<_>>()[0],
        // //     "0xa6c042c7b58b39472428c3adea61e452ac68bf76af8fbff37bb1f5141b24e7c9"
        // // );

        // // serialize agent revision manifests via borsh
        // // let agent_revision_manifests_serialized = borsh::to_vec(&agent_revision_manifests).unwrap();
        // let agent_revision_manifests_serialized = bitcode::encode(&agent_revision);

        // // // compress agent_revision_manifests via zstd
        // // let agent_revision_manifests_compressed = compress_to_vec(&agent_revision_manifests_serialized, 1);
        // // // let mut zstd_encoder = FrameCompressor::new(CompressionLevel::Fastest);
        // // // zstd_encoder.set_drain(Vec::new());
        // // // zstd_encoder.set_source(agent_revision_manifests_serialized);
        // // // zstd_encoder.compress();
        // // // let agent_revision_manifests_compressed: Vec<_> = zstd_encoder.take_drain().unwrap();
        // // // let mut agent_revision_manifests_compressed = Vec::new();
        // // // compress(agent_revision_manifests_serialized, &mut agent_revision_manifests_compressed, CompressionLevel::Fastest);
        // // // // let agent_revision_manifests_compressed = compress_to_vec(agent_revision_manifests_serialized, CompressionLevel::Fastest);
        // // assert_eq!(agent_revision_manifests_serialized.len(), agent.len());

        // // save agent_revision_manifests_serialized onchain
        // self.agent_serialized_manifests_serialized.set_bytes(agent_revision_manifests_serialized);

        // // mark the manifest as latest
        // self.agent_serialized_latest_manifest_hash.set(agent_revision_manifest_hash);

        // let mut existing_chunk_count = 0;
        // let agent_revision_manifest: Vec<U256> = agent_chunks.clone().map(|chunk| {
        //     let chunk_hash = U256::from(chunk.hash);

        //     if self.agent_serialized.get(chunk_hash).is_empty() {
        //         self.agent_serialized
        //             .setter(chunk_hash)
        //             .set_bytes(&agent[chunk.offset..(chunk.offset + chunk.length)]);
        //     } else {
        //         existing_chunk_count += 1;
        //     }

        //     chunk_hash
        // }).collect();

        // // assert_eq!(existing_chunk_count, 0);

        // // u8 * 32 = U256 😅
        // let agent_revision_manifest_hash = keccak(
        //     agent_revision_manifest
        //         .iter()
        //         .flat_map(|hash| hash.to_be_bytes::<32>())
        //         .collect::<Vec<u8>>()
        // ).into();

        // // loop thru each manifest part to build the manifest
        // agent_revision_manifest.iter().for_each(|part|
        //     self.agent_serialized_manifests
        //         .setter(agent_revision_manifest_hash)
        //         .push(*part)
        // );

        // // mark the manifest as latest
        // self.agent_serialized_latest_manifest_hash.set(agent_revision_manifest_hash);
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
    // extern crate cdchunking;

    use std::fs;
    use std::path::PathBuf;
    // use cdchunking::{Chunker, ChunkInput, ZPAQ};
    // use cdc::*;
    // use cbor::{Decoder, Encoder};
    // use base64::engine::{general_purpose, Engine as _};
    use std::io::{self, Cursor, Read, Write}; // Import flush function
    use std::io::BufReader;
    use filebuffer::FileBuffer;
    use md5::{Md5, Digest};
    // use ciborium::{de::from_reader, de::from_reader_with_buffer, ser::into_writer};
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

        let model_rev1_save_start_time = std::time::Instant::now();
        contract.add_agent_revision(model_rev1_buffer.to_vec());
        let model_rev1_save_elapsed_time = model_rev1_save_start_time.elapsed(); // End timer
        println!("Time taken for saving model rev1: {:?}", model_rev1_save_elapsed_time);

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

        let model_rev2_save_start_time = std::time::Instant::now();
        contract.add_agent_revision(model_rev2_buffer.to_vec());
        let model_rev2_save_elapsed_time = model_rev2_save_start_time.elapsed(); // End timer
        println!("Time taken for saving model rev2: {:?}", model_rev2_save_elapsed_time);

        // get the latest model revision
        let get_latest_model_start_time = std::time::Instant::now();
        let latest_agent_revision = contract.get_agent_latest_revision();
        let get_latest_model_elapsed_time = get_latest_model_start_time.elapsed(); // End timer
        assert_eq!(latest_agent_revision, model_rev2_buffer.to_vec(), "Not the same!");
        println!("Time taken for loading the latest model: {:?}", get_latest_model_elapsed_time);

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
//