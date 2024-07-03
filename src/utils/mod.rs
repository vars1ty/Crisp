use ahash::AHashMap;
use parking_lot::RwLock;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::Arc,
};

/// System-related utilities.
#[derive(Default)]
pub struct SystemUtils {
    /// All the listening command outputs.
    /// Each key is the unique identifier, whereas the value is the last-read line.
    listening_command_outputs: Arc<RwLock<AHashMap<String, String>>>,
}

impl SystemUtils {
    /// Executes a command and optionally returns the output if `capture_output` is `true`.
    /// If not, the output will always be `None`.
    pub fn execute(cmd: String, capture_output: bool) -> Option<String> {
        if cmd.is_empty() {
            eprintln!("[ERROR] Empty command input, nothing to return!");
            return None;
        }

        let cmd = Command::new("sh")
            .args(["-c", &cmd])
            .output()
            .unwrap()
            .stdout;

        if !capture_output {
            return None;
        }

        String::from_utf8(cmd).ok().map(|mut result| {
            // Remove trailing \n.
            result.pop();
            result
        })
    }

    /// Starts a new listening command instance.
    pub fn start_listening_command(&self, identifier: String, cmd: String) {
        let listening_command_outputs = self.get_listening_command_outputs();
        std::thread::spawn(move || {
            if listening_command_outputs
                .try_read()
                .expect("[ERROR] listening_command_outputs is locked!")
                .contains_key(&identifier)
            {
                eprintln!("[ERROR] There is already a listening command named \"{identifier}\"!");
                return;
            }

            listening_command_outputs
                .try_write()
                .expect("[ERROR] listening_command_outputs is locked!")
                .insert(identifier.to_owned(), String::default());

            let mut child = Command::new("sh")
                .args(["-c", &cmd])
                .stdout(Stdio::piped())
                .spawn()
                .expect("[ERROR] Failed starting process!");

            let out = child
                .stdout
                .take()
                .expect("[ERROR] Failed taking child stdout!");

            let reader = BufReader::new(out);
            for line in reader.lines() {
                *listening_command_outputs
                    .write()
                    .get_mut(&identifier)
                    .expect(
                        "[ERROR] Listening command wasn't inserted, or was unexpectedly removed!",
                    ) = line.expect("[ERROR] Corrupt UTF-8 String output from process!");
            }

            child.wait().expect("[ERROR] Process wasn't running!");
        });
    }

    /// All the listening command outputs.
    /// Each key is the unique identifier, whereas the value is the last-read line.
    pub fn get_listening_command_outputs(&self) -> Arc<RwLock<AHashMap<String, String>>> {
        Arc::clone(&self.listening_command_outputs)
    }
}
