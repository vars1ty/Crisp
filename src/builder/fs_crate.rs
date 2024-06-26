use rune::Module;

/// Rune module dedicated towards File System operations.
pub struct FileSystemCrate;

impl FileSystemCrate {
    /// Builds the File System Module.
    pub fn build() -> Module {
        let mut built_crate = Module::with_crate("FileSystem")
            .expect("[ERROR] Failed building the FileSystem crate!");
        built_crate
            .function("read", |path: String| std::fs::read_to_string(path))
            .build()
            .unwrap();
        built_crate
            .function("write", |path: String, contents: String| {
                std::fs::write(path, contents)
            })
            .build()
            .unwrap();

        built_crate
    }
}
