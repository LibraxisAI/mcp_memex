//! TUI Configuration Wizard for rmcp_memex
//!
//! Interactive terminal UI for configuring MCP server and host integrations.

mod app;
mod host_detection;
mod ui;

pub use app::{run_wizard, WizardConfig};
pub use host_detection::{detect_hosts, HostDetection, HostFormat, HostKind};
