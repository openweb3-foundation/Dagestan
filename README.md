بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

# DAGESTAN: Directed Acyclic Graph Engine for Succinct Trusted Asynchronous Network - Consensus Engine

Powering Scalable Web3 Solutions on Setheum

Dagestan (DAGESTAN) is an asynchronous and Byzantine fault tolerant consensus protocol aimed at ordering arbitrary messages (transactions). It has been designed to continuously operate even in the harshest conditions: with no bounds on message-delivery delays and in the presence of malicious actors. This makes it an excellent fit for blockchain-related applications. DAGESTAN is bult using [AlephBFT Consensus protocol](https://github.com/Cardinal-Cryptography/AlephBFT).

## Development

### Makefile targets

- `make check`
	- Type check the code, without std feature, excluding tests.
- `make check-tests`
	- Type check the code, with std feature, including tests.
- `make test`
	- Run tests.

### `Cargo.toml`

DAGESTAN use `Cargo.dev.toml` to avoid workspace conflicts with project cargo config. To use cargo commands in DAGESTAN workspace, create `Cargo.toml` by running

- `cp Cargo.dev.toml Cargo.toml`, or
- `make Cargo.toml`, or
- change the command to `make dev-check` etc which does the copy. (For the full list of `make` commands, check `Makefile`)


# Using DAGESTAN

Main projects using DAGESTAN are [Setheum Network](https://github.com/Setheum-Labs/Setheum) and [Khalifa Blockchain](https://github.com/Khalifa-Blockchain/Khalifa)

## Projects using DAGESTAN (A-Z)

- [If you intend or are using DAGESTAN, please add your project here](https://github.com/Setheum-Labs/Dagestan/edit/main/README.md)

- [Setheum Network](https://github.com/Setheum-Labs/Setheum)

## LICENSE

The primary license for DAGESTAN is the Apache 2.0, see [LICENSE](https://github.com/Setheum-Labs/Dagestan/blob/main/LICENSE.md).
