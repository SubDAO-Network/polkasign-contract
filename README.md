# Ink! Contracts

## Install dependencies
reference [https://paritytech.github.io/ink-docs/getting-started/setup](https://paritytech.github.io/ink-docs/getting-started/setup).

## Clone code
Run
```
git clone --recursive https://github.com/Apron-Network/apron-contracts.git
```

## Compile 
Please **use cargo-contract version 0.11**!
```bash
cargo install cargo-contract --vers ^0.11 --force --locked
```
Run `bash ./build.sh`, you can find `.contract` file in `./release` dir.

or

Run
```bash
cargo +nightly contract build
```
in each contract folder

## Test contract
```bash
cd polkasign
cargo test
```

