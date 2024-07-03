mod fs_crate;
mod layer_shell_crate;
pub mod stdext_crate;

use crate::{config::Config, script::ScriptEngine, utils::SystemUtils};
use fs_crate::FileSystemCrate;
use gtk::{gdk::Display, prelude::*, Application, ApplicationWindow, CssProvider, Widget};
use layer_shell_crate::LayerShellCrate;
use parking_lot::Mutex;
use rune::Module;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

/// Wrapper around `ApplicationWindow` which implements `Sync` in an unsafe way.
struct SafeApplicationWindow(pub ApplicationWindow);

/// Wrapper around `Widget` which implements `Send` in an unsafe way.
struct SafeGTKWidget(pub Widget);

// Force-implement traits so that the structures can be accessed through Rune.
// Safety: This should be safe, as Rune runs on the main thread and Crisp does
// ------- not modify widgets nor access them through other threads.
unsafe impl Sync for SafeApplicationWindow {}
unsafe impl Send for SafeGTKWidget {}

type UserWidgets = Arc<Mutex<HashMap<String, SafeGTKWidget>>>;
type CurrentUserWidget = Arc<Mutex<Option<String>>>;

/// UI Builder structure, responsible for holding all functions related to decorating the GTK UI.
#[derive(Default)]
pub struct UIBuilder {
    /// All created user widgets.
    user_widgets: UserWidgets,

    /// The currently focused user widget identifier.
    current_user_widget: CurrentUserWidget,

    script_engine: OnceLock<Arc<ScriptEngine>>,
}

impl UIBuilder {
    /// Builds all the script modules before executing the main script file and presenting the UI.
    pub fn build_modules(&'static self, script_engine: Arc<ScriptEngine>) {
        let script_engine_clone = Arc::clone(&script_engine);
        self.script_engine.get_or_init(|| script_engine_clone);
        let (script_relative_path, script_data) = Config::get_script_information();
        println!("[INFO] Application ID / Namespace will be set to: \"{script_relative_path}\"");
        let app = Application::builder()
            .application_id(script_relative_path.to_owned())
            .build();

        app.connect_startup(|_| self.load_css());
        app.connect_activate(move |app| {
            let application_window: &'static SafeApplicationWindow = Box::leak(Box::new(
                SafeApplicationWindow(ApplicationWindow::builder().application(app).build()),
            ));

            let script_relative_path = script_relative_path.to_owned().leak();

            let mut gtk_module =
                Module::with_crate("GTK").expect("[ERROR] Failed building GTK crate!");
            let mut system_module =
                Module::with_crate("System").expect("[ERROR] Failed building System crate!");
            gtk_module
                .function("set_window_title", move |title: Option<String>| {
                    application_window.0.set_title(title.as_deref());
                })
                .build()
                .unwrap();
            gtk_module
                .function("set_window_height_request", |height| {
                    application_window.0.set_height_request(height)
                })
                .build()
                .unwrap();
            gtk_module
                .function("set_window_width_request", |width| {
                    application_window.0.set_width_request(width)
                })
                .build()
                .unwrap();
            gtk_module
                .function("set_window_default_height", |height| {
                    application_window.0.set_default_height(height)
                })
                .build()
                .unwrap();
            gtk_module
                .function("set_window_default_width", |width| {
                    application_window.0.set_default_width(width)
                })
                .build()
                .unwrap();
            gtk_module
                .function("set_window_resizable", |resizable| {
                    application_window.0.set_resizable(resizable)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_focused_widget", move |identifier: String| {
                    self.set_focused_widget(identifier)
                })
                .build()
                .unwrap();

            gtk_module
                .function("add_vertical_box", |identifier: String, spacing| {
                    self.add_widget(
                        identifier,
                        gtk::Box::new(gtk::Orientation::Vertical, spacing),
                    )
                })
                .build()
                .unwrap();

            gtk_module
                .function("add_horizontal_box", |identifier: String, spacing| {
                    self.add_widget(
                        identifier,
                        gtk::Box::new(gtk::Orientation::Horizontal, spacing),
                    )
                })
                .build()
                .unwrap();

            gtk_module
                .function("add_label", |identifier: String, text: String| {
                    self.add_widget(identifier, gtk::Label::new(Some(&text)))
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_visible", |visible| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_visible(visible)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_opacity", |opacity| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_opacity(opacity)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_hexpand", |expand| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_hexpand(expand)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_vexpand", |expand| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_vexpand(expand)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_gtk_widget_name", |name: String| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_widget_name(&name)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_tooltip_text", |text: Option<String>| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_tooltip_text(text.as_deref())
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_tooltip_markup", |text: Option<String>| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_tooltip_markup(text.as_deref())
                })
                .build()
                .unwrap();

            gtk_module
                .function("get_width", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .width()
                })
                .build()
                .unwrap();

            gtk_module
                .function("get_width_request", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .width_request()
                })
                .build()
                .unwrap();

            gtk_module
                .function("get_height", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .height()
                })
                .build()
                .unwrap();

            gtk_module
                .function("get_height_request", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .height_request()
                })
                .build()
                .unwrap();

            gtk_module
                .function("get_opacity", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .opacity()
                })
                .build()
                .unwrap();

            gtk_module
                .function("can_focus", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .can_focus()
                })
                .build()
                .unwrap();

            gtk_module
                .function("is_focus", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .is_focus()
                })
                .build()
                .unwrap();

            gtk_module
                .function("has_focus", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .has_focus()
                })
                .build()
                .unwrap();

            gtk_module
                .function("can_target", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .can_target()
                })
                .build()
                .unwrap();

            gtk_module
                .function("is_visible", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .is_visible()
                })
                .build()
                .unwrap();

            gtk_module
                .function("is_focusable", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .is_focusable()
                })
                .build()
                .unwrap();

            gtk_module
                .function("scale_factor", || {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .scale_factor()
                })
                .build()
                .unwrap();

            let script_engine_clone = Arc::clone(&script_engine);
            gtk_module
                .function("add_button", move |identifier: String, label: String| {
                    let script_engine_clone = Arc::clone(&script_engine_clone);
                    let button = gtk::Button::with_label(&label);
                    let identifier_clone = identifier.to_owned();

                    button.connect_clicked(move |_| {
                        script_engine_clone.call_on_button_click(&identifier_clone);
                    });
                    self.add_widget(identifier, button);
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_margin_start", move |start| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_margin_start(start)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_margin_end", move |start| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_margin_end(start)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_size_request", move |width, height| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_size_request(width, height)
                })
                .build()
                .unwrap();

            gtk_module
                .function("update_label_text", move |new_text: String| {
                    self.try_get_current_gtk_widget_as::<gtk::Label>(&self.user_widgets.lock())
                        .expect("[ERROR] The widget you are trying to access is not a label!")
                        .set_text(&new_text);
                })
                .build()
                .unwrap();

            gtk_module
                .function("update_button_text", move |new_text: String| {
                    self.try_get_current_gtk_widget_as::<gtk::Button>(&self.user_widgets.lock())
                        .expect("[ERROR] The widget you are trying to access is not a button!")
                        .set_label(&new_text)
                })
                .build()
                .unwrap();

            gtk_module
                .function("set_halign", |align: String| {
                    self.get_current_gtk_widget(&self.user_widgets.lock())
                        .expect("[ERROR] Couldn't get the current widget!")
                        .0
                        .set_halign(match align.as_str() {
                            "Start" => gtk::Align::Start,
                            "Center" => gtk::Align::Center,
                            "End" => gtk::Align::End,
                            "Fill" => gtk::Align::Fill,
                            _ => panic!("[ERROR] Invalid alignment, use Start, Center, Fill or End!"),
                        });
                })
                .build()
                .unwrap();

            script_engine.assign_ui_modules(vec![
                gtk_module,
                LayerShellCrate::build(application_window, script_relative_path),
                system_module,
                FileSystemCrate::build(),
            ]);
            Self::compile_source(Arc::clone(&script_engine), &script_data);
            script_engine
                .call_on_ui_pre_init()
                .expect("[ERROR] VM failed calling on_ui_pre_init!");

            // Add the root box widget, bypassing set_focused_widget's safety checks only this one
            // time.
            let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
            application_window.0.set_child(Some(&widget));
            self.user_widgets
                .try_lock()
                .expect("[ERROR] user_widgets is locked, cannot add root box widget!")
                .insert("root".to_owned(), SafeGTKWidget(widget.into()));
            self.set_focused_widget("root".to_owned());

            script_engine
                .call_main()
                .expect("[ERROR] VM failed calling main!");
            application_window.0.present();
        });

        app.run();
    }

    /// Loads custom CSS from the `STYLESHEET` environment variable, if defined.
    fn load_css(&self) {
        let Ok(stylesheet_path) = std::env::var("STYLESHEET") else {
            return;
        };

        // Load the CSS file and add it to the provider
        let provider = CssProvider::new();
        provider.load_from_string(
            &std::fs::read_to_string(stylesheet_path)
                .expect("[ERROR] Failed reading custom stylesheet!"),
        );

        // Add the provider to the default screen
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("[ERROR] Couldn't connect to a display!"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
        println!("[INFO] Custom CSS loaded!");
    }

    /// Adds a new widget to the UI.
    fn add_widget<W: gtk::prelude::IsA<gtk::Widget>>(&self, identifier: String, widget: W) {
        if !self.can_add_widgets_to_current() {
            eprintln!("[ERROR] Cannot add widgets into a widget that can't hold child widgets!");
            return;
        }

        let Some(mut user_widgets) = self.user_widgets.try_lock() else {
            eprintln!("[ERROR] user_widgets is locked, cannot add widget!");
            return;
        };

        if user_widgets.contains_key(&identifier) {
            eprintln!("[ERROR] Widget \"{identifier}\" already exists!");
            return;
        }

        let downcast = self.try_get_current_gtk_widget_as::<gtk::Box>(&user_widgets);
        if let Some(box_widget) = downcast {
            self.connect_enter_exit_events(identifier.to_owned(), &widget);
            box_widget.append(&widget);
            user_widgets.insert(identifier, SafeGTKWidget(widget.into()));
            drop(user_widgets); // Release Mutex lock.
            return;
        }

        eprintln!("[ERROR] Failed casting widget!");
    }

    /// Connects the Enter and Exit events for a widget, into Rune.
    fn connect_enter_exit_events<W: gtk::prelude::IsA<gtk::Widget>>(
        &self,
        identifier: String,
        widget: &W,
    ) {
        let script_engine = Arc::clone(
            self.script_engine
                .get()
                .expect("[ERROR] No stored Script Engine!"),
        );

        let identifier_clone = identifier.to_owned();
        let script_engine_clone = Arc::clone(&script_engine);
        let motion_controller = gtk::EventControllerMotion::new();
        motion_controller.connect_enter(move |_, _, _| {
            script_engine_clone.call_enter_exit(&identifier_clone, true);
        });

        let identifier_clone = identifier.to_owned();
        let script_engine_clone = Arc::clone(&script_engine);
        motion_controller.connect_leave(move |_| {
            script_engine_clone.call_enter_exit(&identifier_clone, false);
        });
        widget.add_controller(motion_controller);
    }

    /// Switches focus from one widget to another.
    fn set_focused_widget(&self, identifier: String) -> bool {
        let Some(user_widgets) = self.user_widgets.try_lock() else {
            eprintln!(
                "[ERROR] user_widgets is locked, cannot swap focus over to \"{identifier}\"!"
            );
            return false;
        };

        if !user_widgets.contains_key(&identifier) {
            eprintln!("[ERROR] No widget has been defined as \"{identifier}\"!");
            return false;
        }

        let Some(mut current_user_widget) = self.current_user_widget.try_lock() else {
            eprintln!("[ERROR] current_user_widget is locked, cannot swap focus over to \"{identifier}\"!");
            return false;
        };

        *current_user_widget = Some(identifier);
        true
    }

    /// Attempts to downcast the current widget as `W`, returning it as `&W` if successful.
    /// For example:
    /// ```rust
    /// let widget = try_get_current_gtk_widget_as::<gtk::Box>(&self.user_widgets.lock())
    ///     .expect("Error while trying to get the current widget!")
    ///     .expect("Error while trying to cast widget as gtk::Box!");
    /// ```
    fn try_get_current_gtk_widget_as<'a, W: gtk::prelude::IsA<gtk::Widget>>(
        &'a self,
        user_widgets: &'a HashMap<String, SafeGTKWidget>,
    ) -> Option<&W> {
        let Some(current_user_widget) = self.current_user_widget.try_lock() else {
            eprintln!("[ERROR] current_user_widget is locked!");
            return None;
        };

        let Some(current_user_widget) = current_user_widget.as_ref() else {
            eprintln!("[ERROR] current_user_widget is None, there is no focused widget!");
            return None;
        };

        if !user_widgets.contains_key(current_user_widget) {
            eprintln!("[ERROR] There is no widget with the specified name!");
            return None;
        }

        let Some(current_user_widget) = user_widgets.get(current_user_widget) else {
            eprintln!("[ERROR] Couldn't locate any widgets with the name specified in current_user_widget!");
            return None;
        };

        let Some(casted_widget) = current_user_widget.0.downcast_ref::<W>() else {
            eprintln!("[ERROR] Failed casting widget to the desired type!");
            return None;
        };

        Some(casted_widget)
    }

    /// Checks if the current widget can be casted to the desired widget type.
    /// Shortcut for using `try_get_current_gtk_widget_as::<W>(...).is_ok()`.
    fn can_cast_current_widget_to<W: gtk::prelude::IsA<gtk::Widget>>(&self) -> bool {
        let Some(user_widgets) = self.user_widgets.try_lock() else {
            eprintln!("[ERROR] user_widgets is locked, cannot use can_cast_current_widget_to!");
            return false;
        };

        self.try_get_current_gtk_widget_as::<W>(&user_widgets)
            .is_some()
    }

    /// Gets the current GTK Widget wrapped inside of `SafeGTKWidget`.
    fn get_current_gtk_widget<'a>(
        &'a self,
        user_widgets: &'a HashMap<String, SafeGTKWidget>,
    ) -> Option<&SafeGTKWidget> {
        user_widgets.get(
            self.current_user_widget
                .try_lock()
                .expect("[ERROR] CurrentUserWidget is locked!")
                .as_ref()?,
        )
    }

    /// Checks if the current widget can hold child widgets.
    fn can_add_widgets_to_current(&self) -> bool {
        self.can_cast_current_widget_to::<gtk::Box>()
    }

    /// Compiles the `script_data` source.
    fn compile_source(script_engine: Arc<ScriptEngine>, script_data: &str) {
        script_engine
            .run_from_input(script_data)
            .expect("[ERROR] Failed building Script Engine!");
    }
}
