// The contents of this module were originally in a separate crate
// (by the same original author, Robert Lord):
//
// https://crates.io/crates/arena-graph
//
// But since both projects' repositories were archived on 2023-11-22,
// I (Vincent Esche) decided to merge them in an effort to revive them:
// The `arena-graph` crate doesn't seem to have much use on its own,
// consists of basically a single file, and it's easier to maintain them this way.

mod graph;
mod graph_guard;
mod node_guard;
mod node_ptr;

pub use self::{graph::*, graph_guard::*, node_guard::*, node_ptr::*};
