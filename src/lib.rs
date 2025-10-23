#[cfg(not(target_os = "windows"))]
compile_error!("Deject only supports Windows");

mod macros;
pub mod error;
pub mod injector;