#![no_std]
use pinocchio::{no_allocator, nostd_panic_handler, program_entrypoint};
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
nostd_panic_handler!();
no_allocator!();

declare_id!("DYiA6XRsVEr3Zrdq8G9nBrYTsw65CW8NRMR1KEsEFkou");
