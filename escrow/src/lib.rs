#![no_std]
use pinocchio::{entrypoint, nostd_panic_handler};
mod constants;
mod errors;
mod helper;
mod instructions;
pub mod processor;
pub mod states;

use processor::process_instruction;

entrypoint!(process_instruction);
nostd_panic_handler!();
