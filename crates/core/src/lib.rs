//! Core library modules for the process manager.
//!
//! This crate contains the application state, domain types, actions,
//! controller logic, process abstractions, and error definitions used
//! by the terminal user interface.
//!
//! The goal of this crate is to keep business and state logic independent
//! from any concrete UI framework.

pub mod action;
pub mod app;
pub mod command;
pub mod controller;
pub mod error;
pub mod manager;
pub mod process;
pub mod system;
