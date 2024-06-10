//! This crate handles the client side actions for server-client app
//!

use std::io::{self, Write};
use std::fs;
use std::path::Path;
use std::borrow::Cow;
use anyhow::{Result};
use thiserror::Error;

///Define custom error types
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Directory '{0}' is not writable")]
    NotWritable(String),

    #[error("Failed to create directory '{0}'")]
    CreateDirFailed(String),

    #[error("Directory '{0}' already exists")]
    AlreadyExists(String),
}

/// Checks if the folder exists in destination
fn dir_exists(path: &str) -> Result<bool, ClientError> {
    Ok(fs::metadata(path).map(|meta| meta.is_dir()).unwrap_or(false))
}

/// Checks if the PATH is writable
fn is_writable(path: &str) -> Result<bool, ClientError> {
    fs::OpenOptions::new()
        .write(true)
        .create(false)
        .open(path)
        .map(|_| true)
        .or_else(|e| match e.kind() {
            io::ErrorKind::PermissionDenied => Ok(false),
            _ => Err(ClientError::Io(e)),
        })
}

/// Creates directory
pub fn create_dir(path: &str) -> Result<(), ClientError> {
    if dir_exists(path)? {
        if !is_writable(path)? {
            return Err(ClientError::NotWritable(path.to_string()));
        } else {
            return Err(ClientError::AlreadyExists(path.to_string())); // Return AlreadyExists error
        }
    } else {
        // Directory doesn't exist, attempt to create it
        fs::create_dir_all(path).map_err(|_| ClientError::CreateDirFailed(path.to_string()))?;
    }

    // Directory exists and is writable, return Ok(())
    Ok(())
}

/// Check if PATH is a valid file and is readable
pub fn is_valid_file(path: &str) -> Result<bool, ClientError> {
    let path = Path::new(path);

    // Check if the path exists and is a file
    if !path.exists() || !path.is_file() {
        return Ok(false);
    }

    // Check if the file is readable
    match fs::File::open(path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Strips the data into 2 parts
pub fn strip_to_second_space(cow: Cow<str>) -> Cow<str> {
    let s: &str = &cow;
    let mut spaces = s.match_indices(' ').take(2).map(|(index, _)| index);

    if let Some(second_space) = spaces.nth(1) {
        Cow::Owned(s[second_space + 1..].to_string())
    } else {
        cow // If there are less than two spaces, return the original Cow
    }
}

/// Takes input and parses entered command into two parts COMMAND and TEXT
pub fn parse_command() -> Option<(String, String)> {

    print!("Enter command: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let trimmed_input = input.trim();

    if trimmed_input.starts_with(".file") || trimmed_input.starts_with(".image") {
        let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        } else {
            return Some((".text".to_string(), trimmed_input.to_string()));
        }
    } else {
        return Some((".text".to_string(), trimmed_input.to_string()));
    }
}