use std::process::Command;

/// System-related utilities.
pub struct SystemUtils;

impl SystemUtils {
    /// Executes a command and returns the output.
    pub fn execute(cmd: String) -> Option<String> {
        if cmd.is_empty() {
            eprintln!("[ERROR] Empty command input, nothing to return!");
            return None;
        }

        String::from_utf8(
            Command::new("sh")
                .args(["-c", &cmd])
                .output()
                .unwrap()
                .stdout,
        )
        .ok()
        .map(|mut result| {
            // Remove trailing \n.
            result.pop();
            result
        })
    }
}
