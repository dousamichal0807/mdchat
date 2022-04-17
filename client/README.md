# mdchat-client

## Dependencies

**Internal dependencies**

- `mdchat_common`

**External dependencies**

- [`mdcrypt`](https://github.com/dousamichal0807/mdcrypt)

## Compilation

To compile the project:

```sh
# 1. Download the project using Git
git clone https://github.com/dousamichal0807/mdchat.git
# 2. Navigate into the directory
cd mdchat
# 3. Change the branch from `development` to a stable branch, for example:
git checkout -b v0.2.0
# 4. Navigate into client directory
cd client
# 5. Build with Cargo
cargo build --release
```