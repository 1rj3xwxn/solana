use anchor_lang::prelude::*;

#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";

/// 代币金库 authority 的种子。和存 SOL 的 `vault`、`nest` 分开。
#[constant]
pub const POT_SEED: &[u8] = b"pot";

#[constant]
pub const HELLO_WORLD_LAMPORTS: u64 = 1;

#[constant]
pub const MAX_COUNT: u64 = 10;
