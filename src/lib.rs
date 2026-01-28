/*
 *
 *
 *  █████ █████ ██████   ██████  ████████  ████  █████ █████   ████████
 * ░░███ ░░███ ░░██████ ██████  ███░░░░███░░███ ░░███ ░░███   ███░░░░███
 *  ░░███ ███   ░███░█████░███ ░░░    ░███ ░███  ░███  ░███ █░███   ░███
 *   ░░█████    ░███░░███ ░███    ███████  ░███  ░███████████░░█████████
 *    ░░███     ░███ ░░░  ░███   ███░░░░   ░███  ░░░░░░░███░█ ░░░░░░░███
 *     ░███     ░███      ░███  ███      █ ░███        ░███░  ███   ░███
 *     █████    █████     █████░██████████ █████       █████ ░░████████
 *    ░░░░░    ░░░░░     ░░░░░ ░░░░░░░░░░ ░░░░░       ░░░░░   ░░░░░░░░
 *
 *                   (c) vw.dvw 2026, MIT or Apache-2.0
 *
*/

//! Abstraction layer for YM2149-adjacent sound chips.
//!
//! The core crate contains ...
//!
//! **When in doubt, check the specsheet!**
// lib.rs is just some glue code... go check out the other files, they're much more interesting!

#![no_std]

pub mod audio;
pub mod chip;
pub mod command;
pub mod envelopes;
pub mod errors;
pub mod io;
pub mod register;
