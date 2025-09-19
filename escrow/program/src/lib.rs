#![no_std]
use pinocchio::{entrypoint, nostd_panic_handler};
mod instructions;
mod processor;
mod states;

use processor::process_instruction;

entrypoint!(process_instruction);
nostd_panic_handler!();
