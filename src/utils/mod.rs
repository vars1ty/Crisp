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
    /// Each value is wrapped inside RwLock<String> so that it's easier to use it in multiple
    /// background loops, without blocking the entire HashMap when writing to a single key.
    listening_command_outputs: Arc<RwLock<AHashMap<String, RwLock<String>>>>,
}

impl SystemUtils {
    /// Executes a command and optionally returns the output if `capture_output` is `true`.
    /// If not, the output will always be `None`.
    pub fn execute(cmd: String, capture_output: bool) -> Option<String> {
        if cmd.is_empty() {
            eprintln!("[ERROR] Empty command input, nothing to return!");
            return None;
        }

        let command = Command::new("sh").args(["-c", &cmd]).output();
        if command.is_err() {
            eprintln!(
                "[ERROR] Failed spawning \"sh -c {cmd}\", error: {}",
                command.unwrap_err()
            );
            return None;
        }

        if !capture_output {
            return None;
        }

        String::from_utf8(
            command
                .expect("[FATAL] Safety if-condition removed for process spawning?")
                .stdout,
        )
        .ok()
        .map(|mut result| {
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
                .insert(identifier.to_owned(), RwLock::default());

            let child = Command::new("sh")
                .args(["-c", &cmd])
                .stdout(Stdio::piped())
                .spawn();
            if child.is_err() {
                eprintln!(
                    "[ERROR] Failed spawning \"sh -c {cmd}\", error: {}",
                    child.unwrap_err()
                );
                return;
            }

            let mut child =
                child.expect("[FATAL] Safety if-condition removed for child process spawning?");

            let Some(out) = child.stdout.take() else {
                eprintln!("[ERROR] Child process has no stdout to aquire!");
                return;
            };

            let reader = BufReader::new(out);
            for line in reader.lines() {
                let Some(reader) = listening_command_outputs.try_read() else {
                    eprintln!("[ERROR] listening_command_outputs is locked!");
                    continue;
                };

                let Some(command) = reader.get(&identifier) else {
                    eprintln!("[ERROR] There is no listening command named \"{identifier}\"!");
                    return;
                };

                let Some(mut writer) = command.try_write() else {
                    eprintln!("[ERROR] The listening command \"{identifier}\" cannot be written to as its locked!");
                    continue;
                };

                *writer = line.expect("[ERROR] Corrupt UTF-8 String output from process!");
            }

            if let Err(error) = child.wait() {
                eprintln!("[ERROR] Child process exited, error: {error}");
            }
        });
    }

    /// All the listening command outputs.
    /// Each key is the unique identifier, whereas the value is the last-read line.
    pub fn get_listening_command_outputs(&self) -> Arc<RwLock<AHashMap<String, RwLock<String>>>> {
        Arc::clone(&self.listening_command_outputs)
    }
}
