# MDChat Server

A simple chat server written in Rust.

## Dependencies

- Rust's Cargo 
- [mdchat-util](https://github.com/dousamichal0807/mdchat-util) (automatically downloaded by Cargo)

## Compilation

To compile the project:

```shell
# 1. Download the project using Git
git clone https://github.com/dousamichal0807/mdchat-server.git
# 2. Navigate into the directory
cd mdchat-server
# 3. Change the branch from `development` to a stable branch, for example:
git checkout -b v0.1.0
# 4. Build with Cargo
cargo build --release
```

## Configuration

> For full list configuration options see [this page](doc/configuration.md).