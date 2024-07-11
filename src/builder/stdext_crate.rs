use crate::{script::ScriptEngine, utils::SystemUtils};
use rune::{runtime::Function, Module, Value};
use std::sync::Arc;

/// Rune module for adding new "standard" functions.
pub struct STDExtCrate;

impl STDExtCrate {
    /// Builds the Layer Shell Module.
    pub fn build(system_utils: Arc<SystemUtils>, script_engine: Arc<ScriptEngine>) -> Module {
        let mut built_crate =
            Module::with_crate("std").expect("[ERROR] Failed building the std crate!");
        built_crate
            .function("vtos", |value: Value| format!("{value:?}"))
            .build()
            .unwrap();

        built_crate
            .function("ftoi", |f: f32| f as i32)
            .build()
            .unwrap();
        built_crate
            .function("dtoi", |f: f64| f as i32)
            .build()
            .unwrap();

        built_crate
            .function("itol", |i: i32| i as i64)
            .build()
            .unwrap();
        built_crate
            .function("itof", |i: i32| i as f32)
            .build()
            .unwrap();
        built_crate
            .function("itod", |i: i32| i as f64)
            .build()
            .unwrap();

        built_crate
            .function("ltoi", |l: i64| l as i32)
            .build()
            .unwrap();
        built_crate
            .function("ltof", |l: i64| l as f32)
            .build()
            .unwrap();
        built_crate
            .function("ltod", |l: i64| l as f64)
            .build()
            .unwrap();

        built_crate
            .function("stoi", |s: String| s.parse::<i32>())
            .build()
            .unwrap();

        built_crate
            .function("get_command_output", |cmd| SystemUtils::execute(cmd, true))
            .build()
            .unwrap();

        built_crate
            .function("execute_command", |cmd| SystemUtils::execute(cmd, false))
            .build()
            .unwrap();

        let system_utils_clone = Arc::clone(&system_utils);
        built_crate
            .function("start_listening_command", move |identifier, command| {
                system_utils_clone.start_listening_command(identifier, command);
            })
            .build()
            .unwrap();

        built_crate
            .function("get_listening_command_output", move |identifier: String| {
                let cmd_outputs = system_utils.get_listening_command_outputs();
                let Some(reader) = cmd_outputs.try_read() else {
                    eprintln!("[ERROR] listening_command_outputs is locked, returning None!");
                    return None;
                };

                reader
                    .get(&identifier)
                    .and_then(|res| res.try_read().map(|res| res.to_owned()))
            })
            .build()
            .unwrap();

        built_crate
            .function(
                "start_background_loop",
                move |identifier: String, loop_function: Function, time| {
                    let loop_function_dbg_string = format!("{loop_function:?}");
                    let loop_function = loop_function
                        .into_sync()
                        .into_result()
                        .expect("[ERROR] Function cannot be turned into a SyncFunction!");
                    script_engine.start_background_loop(
                        format!("{identifier} [{loop_function_dbg_string}]"),
                        loop_function,
                        time,
                    );
                },
            )
            .build()
            .unwrap();

        built_crate
    }
}
