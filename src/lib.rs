#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
#![feature(drain_filter)]
#![feature(vec_remove_item)]
#![recursion_limit = "128"]

#[macro_use]
pub mod macros;
pub mod analysis;
pub mod codegen;
pub mod exec;
pub mod ir;
pub mod traits;
pub mod util;

pub use ir::*;

extern crate defs;
#[cfg(feature = "x86_64")]
#[macro_use]
extern crate dynasm;
extern crate dynasmrt;
#[macro_use]
extern crate target_lexicon;
extern crate faerie;
extern crate id_arena;
#[macro_use]
extern crate lazy_static;
extern crate rustc_hash;

pub use rustc_hash::{FxHashMap, FxHashSet};
