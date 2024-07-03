use super::SafeApplicationWindow;
use gtk4_layer_shell::*;
use rune::Module;

/// Rune module for managing the layer shell.
pub struct LayerShellCrate;

impl LayerShellCrate {
    /// Builds the Layer Shell Module.
    pub fn build(
        application_window: &'static SafeApplicationWindow,
        script_relative_path: &'static str,
    ) -> Module {
        let mut built_crate = Module::with_crate("LayerShell")
            .expect("[ERROR] Failed building the LayerShell crate!");
        built_crate
            .function(
                "init_layer_shell",
                move |enable_exclusive_zone, layer: String| {
                    application_window.0.init_layer_shell();
                    application_window.0.set_namespace(script_relative_path);
                    if enable_exclusive_zone {
                        application_window.0.auto_exclusive_zone_enable();
                    }

                    match layer.as_str() {
                        "Top" => application_window.0.set_layer(Layer::Top),
                        "Bottom" => application_window.0.set_layer(Layer::Bottom),
                        "Overlay" => application_window.0.set_layer(Layer::Overlay),
                        "Background" => application_window.0.set_layer(Layer::Background),
                        _ => panic!("[ERROR] Invalid layer value!"),
                    }
                },
            )
            .build()
            .unwrap();

        built_crate
            .function("set_anchors", |left, right, top, bottom| {
                let anchors = [
                    (Edge::Left, left),
                    (Edge::Right, right),
                    (Edge::Top, top),
                    (Edge::Bottom, bottom),
                ];

                for (anchor, state) in anchors {
                    application_window.0.set_anchor(anchor, state);
                }
            })
            .build()
            .unwrap();

        built_crate
    }
}
