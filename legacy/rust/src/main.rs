//! NotAlterra — Subnautica 2 save-file manager (Windows GUI)
//!
//! Native Windows 11 application built on the `windows` crate
//! with WinUI 3.  No XAML compiler needed — UI is built in Rust.
//!
//! MIT License.  Not affiliated with Unknown Worlds Entertainment or KRAFTON.

#![windows_subsystem = "windows"]

use anyhow::Result as AnyResult;
use std::cell::RefCell;
use windows::core::*;
use windows::UI::Xaml::*;

/// Main entry point — creates the WinUI Application and
/// starts the Windows message pump.
fn main() -> Result<()> {
    Application::Start(|_| {
        // Application callback — create the main window
        if let Err(e) = create_window() {
            // Log startup failure
            eprintln!("Failed to create window: {e}");
        }
    })
}

/// Create the main application window with WinUI content.
fn create_window() -> Result<()> {
    let window = Window::new()?;
    window.SetTitle(&HSTRING::from("NotAlterra"))?;

    // TODO: Build the WinUI control tree
    // let stack = StackPanel::new()?;
    // window.SetContent(&stack)?;

    window.Activate()?;
    Ok(())
}
