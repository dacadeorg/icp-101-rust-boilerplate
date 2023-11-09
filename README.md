# voting_system

## Description:
This smart contract, developed using the Internet Computer Protocol (ICP), facilitates a robust and transparent voting system. It empowers users to engage in diverse voting activities, ensuring the integrity and security of the entire process. With a decentralized and tamper-proof structure, this contract enables seamless execution of various election types, polls, and decision-making events, all within a secure and trusted environment.

## Features:
- Easy and secure addition of votes to the system.
- Efficient retrieval of votes based on specific requirements.
- Seamless access to the latest vote timestamp for effective tracking and analysis.
- Reliable deletion of votes when necessary, ensuring data integrity and management.
- Support for diverse types of elections and polls, accommodating varying voting scenarios.
- Robust data management and storage mechanisms, ensuring the integrity and security of voting data.
- Customizable settings and configurations, allowing flexibility in the voting process based on specific requirements and use cases.

## Tech Stack:
- Programming Language: Rust
- Framework: Internet Computer Protocol (ICP)
- Data Storage: Stable Memory and Stable BTreeMap
- Web Assembly Target: wasm32-unknown-unknown
- Libraries: candid, serde, ic-cdk
- Development Tools: rustc, cargo, dfx
- Version Control: Git

## Installation

### Requirements
Ensure you have the following installed in your development environment:

* rustc 1.64 or higher
```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
cargo install candid-extractor
```
* install `dfx`
```bash
DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
source ~/.bashrc
dfx start --background
```

### Install Locally
To start working on your project, you can use the following commands:

```bash
git clone https://github.com/Ganzzi/Voting-System-ICP.git
cd voting_system/
chmod +x did.sh
npm run gen-deploy
```

Interact with the Canister (add_vote & get_votes for examle):

```bash
dfx canister call votitng_system add_vote '("iamuser1", "iamuser2")'
dfx canister call votitng_system get_votes
```

and any other functions
