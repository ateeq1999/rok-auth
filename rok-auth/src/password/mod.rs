//! Password hashing utilities using Argon2id.

mod hash;

pub use hash::{hash, hash_async, verify, verify_async};
