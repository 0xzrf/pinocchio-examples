#![no_std]
use pinocchio::{default_allocator, nostd_panic_handler, program_entrypoint};
use pinocchio_pubkey::declare_id;
use processor::process_instruction;

pub mod errors;
pub use errors::*;
mod constants;
mod helpers;
pub use helpers::*;
mod instructions;
pub mod processor;
pub mod states;

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

declare_id!("DYiA6XRsVEr3Zrdq8G9nBrYTsw65CW8NRMR1KEsEFkou");
