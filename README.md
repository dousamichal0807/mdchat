# MDChat

GitHub repository for MDChat. This repository contains both client and server part of the application. For more information about provided CLI client see [`client` folder documentation](client/README.md). For more about the server see [`server` folder documentation](server/README.md).

## How to read subprojects' documentation

Each subproject in this repository contains its own documentation. Main points of documentation are:

### Dependencies

Dependencies are divided into *internal* and *external*.

Internal dependencies represent dependencies, which are contained in this repository whereas external dependencies are dependencies from other git repositories (which are not included in https://crates.io registry).

Dependencies from Rust's official Cargo registry (https://crates.io) are not mentioned in the documentation. See subproject's `Cargo.toml` and (if present) `Cargo.lock` files for all dependencies of the subproject.

### Compilation manual

For [`mdchat_client`](client/README.md) and [`mdchat_server`](server/README.md) a simple manual for compilation is included in the documentation. There is no need to compile dependencies since Rust's Cargo compiles them automatically.

### Subproject-specific information

Contains (if any) information how the subproject is bundled into another subproject and what is its function.

## License

If not stated otherwise, all parts of this repository are licensed under GNU Affero General Public License v3 or higher. See [LICENSE](LICENSE) file for more information.