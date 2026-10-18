//! LLM provider adapters. Keys are passed in by the caller from the keychain
//! and never leave this process except in requests to the chosen provider.

pub mod ollama;
