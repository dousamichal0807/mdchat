# MDChat Server

A simple chat server written in Rust.

## Dependencies

**Internal dependencies**

- `mdchat_common`
- `mdchat_serverconf`

**External dependencies**

- [`mdcrypt`](https://github.com/dousamichal0807/mdcrypt)
- [`mdlog`](https://github.com/dousamichal0807/mdlog)

## Compilation

To compile the project:

```shell
# 1. Download the project using Git
git clone https://github.com/dousamichal0807/mdchat.git
# 2. Navigate into the directory
cd mdchat
# 3. Change the branch from `development` to a stable branch, for example:
git checkout -b v0.2.0
# 4. Navigate into server directory
cd server
# 5. Build with Cargo
cargo build --release
```

## Configuration

> For full list configuration options see [this page](../serverconf/README.md).