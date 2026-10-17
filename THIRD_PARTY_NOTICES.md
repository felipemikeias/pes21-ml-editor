# Third-party notices

## pesXdecrypter

`crates/core/src/crypto.rs` and `crates/core/src/mt19937.rs` re-implement in Rust the save
cipher of [pesXdecrypter](https://github.com/the4chancup/pesXdecrypter) and the master keys
it publishes. pesXdecrypter is released into the public domain under the Unlicense
(<https://unlicense.org>). The MT19937 algorithm is by Makoto Matsumoto and Takuji Nishimura.

## Rust crates

Dependencies and their licenses are listed in `Cargo.lock`; run
`cargo tree --workspace` for the full graph.
