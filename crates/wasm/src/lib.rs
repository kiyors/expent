pub mod aggregations;
pub mod budget;
pub mod currency;
pub mod financial;
pub mod matching;
pub mod parsing;
pub mod text;
pub mod validation;

pub use aggregations::*;
pub use budget::*;
pub use currency::*;
pub use financial::*;
pub use matching::*;
pub use parsing::*;
pub use text::*;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
