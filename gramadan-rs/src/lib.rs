pub mod features;
pub mod opers;
pub mod singular_info;
pub mod plural_info;
pub mod adjective;
pub mod noun;
pub mod np;
pub mod verb;
pub mod enrich;

#[cfg(feature = "python")]
mod python;
