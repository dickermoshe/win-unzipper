#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{self, Command};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    
    if !input_path.exists() {
        process::exit(1);
    }

    let resolved_path = match fs::canonicalize(&input_path) {
        Ok(p) => p,
        Err(_) => {
            process::exit(1);
        }
    };

    // Create output directory: parent/filename_without_extension
    let parent = resolved_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_stem = resolved_path.file_stem().unwrap_or_default();
    let out_dir = parent.join(file_stem);

    if !out_dir.exists() {
        let out_arg = format!("-o{}", out_dir.display());
        
        match Command::new("7z")
            .args(&["x", &resolved_path.to_string_lossy(), &out_arg, "-y"])
            .creation_flags(CREATE_NO_WINDOW)
            .output() 
        {
            Ok(output) if output.status.success() => {
                // Extraction successful
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                show_message_box(&format!("Extraction failed:\n{}", stderr), "Unzipper");
                process::exit(output.status.code().unwrap_or(1));
            }
            Err(e) => {
                show_message_box(&format!("Failed to run 7z: {}", e), "Unzipper");
                process::exit(1);
            }
        }
    }

    // Open Explorer to the extracted folder
    let _ = Command::new("explorer.exe").arg(&out_dir).spawn();
}



fn show_message_box(message: &str, title: &str) {
    // Convert to null-terminated UTF-16 (wide strings)
    let message_wide: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title_wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    
    unsafe {
        MessageBoxW(
            None, 
            windows::core::PCWSTR(message_wide.as_ptr()),
            windows::core::PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONINFORMATION
        );
    }
}