#!/usr/bin/env nix
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.95"
//! chrono = "0.4.39"
//! [dependencies.uuid]
//! version = "1.12.1"
//! features = [
//!   "v4",
//!   "fast-rng",
//! ]
//! ```
/*
#!nix shell nixpkgs#clang nixpkgs#cargo nixpkgs#rustc nixpkgs#rust-script --command rust-script
*/

use anyhow::{Result, Context};
use uuid::Uuid;
use std::fs;
use std::process::Command;
use std::os::unix::process::CommandExt;
use chrono::prelude::*;
use std::io::Write;

fn main() -> Result<()> {
    let timestamp = Utc::now();
    let id = Uuid::new_v4();
    let filename = format!("content/µ/{id}.md");

    let mut file = fs::File::create(&filename)
        .with_context(|| format!("failed to create '{filename}'"))?;

    let content = format!(
r##"+++
date = {}
draft = true
template = "pages.html"
+++

"##, timestamp.format("%Y-%m-%d %H:%M:%S"));

    file.write_all(content.as_bytes())
        .with_context(|| format!("failed to write template to new post '{filename}'"))?;
    Err(Command::new("hx").arg(&filename).exec())
        .with_context(|| format!("failed to exec Helix on '{filename}'"))?
}
