use crate::{builder::stdext_crate::STDExtCrate, utils::SystemUtils};
use rune::{
    runtime::{Function, SyncFunction, Value},
    termcolor::*,
    *,
};
use std::{
    cell::{OnceCell, RefCell},
    sync::Arc,
};

/// Data collected from the `main` function return data.
#[derive(Any)]
struct MainReturnData {
    /// Single argument to be passed into every event function, such as a structure instance.
    pub event_arg: &'static SafeValue,

    /// `on_button_click` Rune function.
    pub on_button_click_rfn: Function,

    /// `on_enter_widget` Rune function.
    pub on_enter_widget_rfn: Function,

    /// `on_exit_widget` Rune function.
    pub on_exit_widget_rfn: Function,

    /// Background loops registered.
    /// 0 -> The function to be called.
    /// 1 -> How often (in milliseconds) the function should be called.
    pub background_loops: &'static Vec<(SyncFunction, u64)>,
}

/// Script Engine active for this particular instance.
#[derive(Default)]
pub struct ScriptEngine {
    /// The allocated Rune Virtual Machine.
    rune_vm: OnceCell<RefCell<Vm>>,

    /// Rune UI module to be installed.
    ui_modules: OnceCell<Vec<Module>>,

    /// Data collected from the `main` function return data.
    main_return_data: OnceCell<MainReturnData>,

    /// System Utils instance.
    system_utils: Arc<SystemUtils>,
}

/// Hacked "thread-safe" `Value` wrapper.
#[derive(Debug, Any)]
struct SafeValue(pub Value);
unsafe impl Send for SafeValue {}
unsafe impl Sync for SafeValue {}

unsafe impl Send for ScriptEngine {}
unsafe impl Sync for ScriptEngine {}

impl ScriptEngine {
    /// Builds a new virtual machine from the source `input` and then calls the `main` function on the
    /// source.
    pub fn run_from_input(&self, input: &str) -> rune::support::Result<()> {
        let mut context = Context::with_default_modules()?;
        let mut module = Module::new();
        module.ty::<MainReturnData>()?;
        module
            .function(
                "new_runtime_config",
                |event_arg,
                 on_button_click_rfn,
                 on_enter_exit_widget_rfn: (Function, Function),
                 background_loops| {
                    MainReturnData {
                        event_arg: Box::leak(Box::new(SafeValue(event_arg))),
                        on_button_click_rfn,
                        on_enter_widget_rfn: on_enter_exit_widget_rfn.0,
                        on_exit_widget_rfn: on_enter_exit_widget_rfn.1,
                        background_loops: Box::leak(Box::new(background_loops)),
                    }
                },
            )
            .build()
            .unwrap();
        context.install(STDExtCrate::build(Arc::clone(&self.system_utils)))?;
        context.install(module)?;

        for module in self
            .ui_modules
            .get()
            .expect("[ERROR] Missing custom modules!")
        {
            context.install(module)?;
        }

        let mut sources = Sources::new();
        sources.insert(Source::memory(input)?)?;

        let mut diagnostics = Diagnostics::new();
        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            diagnostics.emit(&mut StandardStream::stderr(ColorChoice::Auto), &sources)?;
        }

        let vm = Vm::new(Arc::new(context.runtime()?), Arc::new(result?));
        self.rune_vm.get_or_init(|| RefCell::new(vm));
        Ok(())
    }

    /// Calls the `main` function on the VM.
    pub fn call_main(&self) -> rune::support::Result<()> {
        let mut vm = self
            .rune_vm
            .get()
            .expect("[ERROR] No VM has been stored!")
            .try_borrow_mut()
            .expect("[ERROR] VM is already being borrowed, cannot borrow as mutable!");

        let result = vm.call(["main"], ())?;

        if let Some(data) = rune::from_value::<Option<MainReturnData>>(result)? {
            for (func, time) in data.background_loops {
                gtk::glib::timeout_add(std::time::Duration::from_millis(*time), move || {
                    func.call::<_, ()>((&data.event_arg.0,))
                        .into_result()
                        .expect("[ERROR] Failed calling background loop!");
                    gtk::glib::ControlFlow::Continue
                });
            }

            self.main_return_data.get_or_init(|| data);
        }

        Ok(())
    }

    /// Calls the `on_ui_pre_init` function on the VM if found.
    pub fn call_on_ui_pre_init(&self) -> rune::support::Result<()> {
        let vm = self
            .rune_vm
            .get()
            .expect("[ERROR] No VM has been stored!")
            .try_borrow()
            .expect("[ERROR] VM is already being borrowed, cannot borrow as of now!");

        let Ok(ui_pre_init) = vm.lookup_function(["on_ui_pre_init"]) else {
            println!("[WARN] No on_ui_pre_init function, skipping.");
            return Ok(());
        };

        ui_pre_init.call(()).into_result()?;
        Ok(())
    }

    pub fn assign_ui_modules(&self, modules: Vec<Module>) {
        self.ui_modules.get_or_init(|| modules);
    }

    /// Calls the `on_enter_widget` or `on_exit_widget` function on the VM if present.
    pub fn call_enter_exit(&self, identifier: &str, entered: bool) -> bool {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[WARN] No event functions were stored at startup, skipping advanced events!"
            );
            return false;
        };

        if entered {
            return main_return_data
                .on_enter_widget_rfn
                .call::<_, ()>((&main_return_data.event_arg.0, identifier))
                .into_result()
                .is_ok();
        }

        main_return_data
            .on_exit_widget_rfn
            .call::<_, ()>((&main_return_data.event_arg.0, identifier))
            .into_result()
            .is_ok()
    }

    /// Calls the `on_tick` function on the active VM if present.
    pub fn call_on_button_click(&self, identifier: &str) {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[WARN] No event functions were stored at startup, skipping advanced events!"
            );
            return;
        };

        main_return_data
            .on_button_click_rfn
            .call::<_, ()>((&main_return_data.event_arg.0, identifier))
            .into_result()
            .expect("[ERROR] VM failed to call on_button_click!");
    }
}
