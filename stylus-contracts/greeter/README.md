# Stylus Greeter: Onchain AI Agent

A proof of concept implementation of an AI-powered greeter agent using the Arbitrum Stylus SDK in Rust. This project demonstrates how to create, deploy and interact with an intelligent greeting contract on Arbitrum's Stylus chain, showcasing the potential of running AI agents directly onchain.

## Quick Start

Install [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo

```bash
cargo install --force cargo-stylus cargo-stylus-check
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```
rustup target add wasm32-unknown-unknown
```

You should now have it available as a Cargo subcommand:

```bash
cargo stylus --help
```

Then, clone the template:

```
git clone https://github.com/OffchainLabs/stylus-hello-world && cd stylus-hello-world
```

### Build and Test

Before deploying, you can build and test your contract locally:

```bash
# Build the contract
cargo build --release

# Run the tests
cargo test
```

### ABI Export

Generate the Solidity ABI for the greeter program:

```bash
cargo stylus export-abi
```

This will output the contract interface showing all available public functions.

## Deployment

You can use the `cargo stylus` command to also deploy your program to the Stylus testnet. We can use the tool to first check
our program compiles to valid WASM for Stylus and will succeed a deployment onchain without transacting. By default, this will use the Stylus testnet public RPC endpoint. See here for [Stylus testnet information](https://docs.arbitrum.io/stylus/reference/testnet-information)

```bash
cargo stylus check
```

If successful, you should see:

```bash
Finished release [optimized] target(s) in 1.88s
Reading WASM file at stylus-hello-world/target/wasm32-unknown-unknown/release/stylus-hello-world.wasm
Compressed WASM size: 8.9 KB
Program succeeded Stylus onchain activation checks with Stylus version: 1
```

Next, we can estimate the gas costs to deploy and activate our program before we send our transaction. Check out the [cargo-stylus](https://github.com/OffchainLabs/cargo-stylus) README to see the different wallet options for this step:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> \
  --estimate-gas
```

You will then see the estimated gas cost for deploying before transacting:

```bash
Deploying program to address e43a32b54e48c7ec0d3d9ed2d628783c23d65020
Estimated gas for deployment: 1874876
```

The above only estimates gas for the deployment tx by default. To estimate gas for activation, first deploy your program using `--mode=deploy-only`, and then run `cargo stylus deploy` with the `--estimate-gas` flag, `--mode=activate-only`, and specify `--activate-program-address`.


Here's how to deploy:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH>
```

The CLI will send 2 transactions to deploy and activate your program onchain.

```bash
Compressed WASM size: 8.9 KB
Deploying program to address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 1973450
Submitting tx...
Confirmed tx 0x42db…7311, gas used 1973450
Activating program at address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 14044638
Submitting tx...
Confirmed tx 0x0bdb…3307, gas used 14044638
```

Once both steps are successful, you can interact with your program as you would with any Ethereum smart contract.

## Interacting with the Contract

You can interact with your deployed contract using any Ethereum tooling since it's compatible with the Ethereum ABI. Here's how to interact using Rust:

1. Set up your environment variables in `.env`:
```
RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
STYLUS_CONTRACT_ADDRESS=<your_deployed_contract_address>
PRIV_KEY_PATH=<path_to_private_key>
```

2. Run the example code:
```bash
cargo run --example greeter --target=<YOUR_ARCHITECTURE>
```

You can find your architecture by running `rustc -vV | grep host`.

## Build Options

The contract can be built with different optimization levels using cargo-stylus. For size optimizations and other build options, see the [cargo-stylus documentation](https://github.com/OffchainLabs/cargo-stylus/blob/main/OPTIMIZING_BINARIES.md).

## License

Licensed under either of Apache-2.0 or MIT license at your option.
