mod builder;
mod config;
mod errors;
mod script;
mod utils;

use builder::UIBuilder;

fn main() {
    let ui_builder: &'static UIBuilder = Box::leak(Box::default());
    ui_builder.build_modules(Default::default());
}
