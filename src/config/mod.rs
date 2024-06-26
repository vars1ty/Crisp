/// Config-related functions.
pub struct Config;

impl Config {
    /// Gets the information from the `SCRIPT_FILE` environment variable.
    /// The tuple consists of 2 strings:
    /// 0 -> The relative file path.
    /// 1 -> Whether to display as a layershell or not.
    /// 2 -> The content of the file.
    pub fn get_script_information() -> (String, bool, String) {
        let env_value = std::env::var("SCRIPT_FILE")
            .expect("[ERROR] No SCRIPT_FILE=\"relative_path\" environment variable found!");
        let layershell = std::env::var("CRISP_AS_LAYERSHELL").unwrap_or_default() == "1";
        let file_data = std::fs::read_to_string(&env_value)
            .expect("[ERROR] Couldn't read the relative path defined in SCRIPT_FILE!");
        (env_value, layershell, file_data)
    }
}
