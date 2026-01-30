// Example demonstrating readline support with command history

use anyhow::Result;

// Use the input handler from the CLI module
use shammah::cli::InputHandler;

fn main() -> Result<()> {
    println!("Readline Demo - Testing keyboard shortcuts");
    println!("=========================================");
    println!();
    println!("Available shortcuts:");
    println!("  Arrow Up/Down    - Navigate command history");
    println!("  Arrow Left/Right - Move cursor within line");
    println!("  Ctrl+A           - Jump to beginning of line");
    println!("  Ctrl+E           - Jump to end of line");
    println!("  Ctrl+K           - Delete from cursor to end of line");
    println!("  Ctrl+U           - Delete entire line");
    println!("  Ctrl+W           - Delete word before cursor");
    println!("  Ctrl+R           - Reverse search through history");
    println!("  Ctrl+C or Ctrl+D - Exit");
    println!();
    println!("Type some commands to test (they'll be saved to ~/.shammah/history.txt)");
    println!();

    let mut handler = InputHandler::new()?;
    let mut command_count = 0;

    loop {
        match handler.read_line(&format!("demo[{}]> ", command_count))? {
            Some(line) => {
                command_count += 1;
                println!("  You typed: {:?}", line);
                println!("  Length: {} characters", line.len());

                // Echo special commands
                match line.trim() {
                    "quit" | "exit" => {
                        println!("Exiting...");
                        break;
                    }
                    "history" => {
                        println!("History is saved in ~/.shammah/history.txt");
                        println!("(Use arrow keys to navigate through it)");
                    }
                    _ => {}
                }
            }
            None => {
                println!();
                println!("Received Ctrl+C or Ctrl+D - exiting gracefully");
                break;
            }
        }
    }

    println!();
    println!("Saving history...");
    handler.save_history()?;
    println!("History saved successfully to ~/.shammah/history.txt");
    println!();
    println!("Try running this demo again - your history will be preserved!");

    Ok(())
}
