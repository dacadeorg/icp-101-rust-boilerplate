# icp_101_rust_boilerplate

This Rust-based boilerplate features a Docker-based development environment, streamlining setup and dependency management. The boilerplate is optimized for use in GitHub Codespaces, allowing for an efficient and seamless development experience.

## Development Environment

The project is pre-configured with all necessary dependencies, including:
- Rust environment
- wasm32-unknown-unknown target
- Candid-extractor
- DFINITY SDK

## Getting Started

### Start a Codespace

   - Open the project in a GitHub Codespace or in VS Code with the Remote-Containers extension.
   - The devcontainer will automatically build and configure the environment.

### Develop and Test

- **Develop**: Start modifying the project and running commands within the devcontainer terminal.
- **Test**: Utilize the provided `Makefile` commands for various operations.

## Makefile Commands

Utilize these commands for efficient project operations:

- **Generate Candid Interface**
    ```bash
    make generate
    ```
    This command generates the Candid interface definitions for your canisters.

- **Deploy Canister on the Playground**
    ```bash
    make gen-deploy (or just make)
    ```
    Deploy your canister to the ICP playground, a temporary mainnet environment for testing, valid for 20 minutes.
