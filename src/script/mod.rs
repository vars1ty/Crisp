use crate::{builder::stdext_crate::STDExtCrate, utils::SystemUtils};
use rune::{
    runtime::{Function, SyncFunction, Value},
    termcolor::*,
    *,
};
use std::{
    cell::{OnceCell, RefCell},
    sync::{Arc, OnceLock},
};

/// Data collected from the `main` function return data.
#[derive(Any)]
struct MainReturnData {
    /// Single argument to be passed into every event function, such as a structure instance.
    pub event_arg: &'static SafeValue,

    /// `on_enter_widget` Rune function.
    pub on_enter_widget_rfn: SyncFunction,

    /// `on_exit_widget` Rune function.
    pub on_exit_widget_rfn: SyncFunction,
}

/// Script Engine active for this particular instance.
#[derive(Default)]
pub struct ScriptEngine {
    /// The allocated Rune Virtual Machine.
    rune_vm: OnceCell<RefCell<Vm>>,

    /// Rune UI module to be installed.
    ui_modules: OnceCell<Vec<Module>>,

    /// Data collected from the `main` function return data.
    main_return_data: Arc<OnceLock<MainReturnData>>,

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
    pub fn run_from_input(&self, input: &str, self_arc: Arc<Self>) -> rune::support::Result<()> {
        let mut context = Context::with_default_modules()?;
        let mut module = Module::new();
        module.ty::<MainReturnData>()?;

        let main_return_data_clone = Arc::clone(&self.main_return_data);
        module
            .function(
                "init_runtime_config",
                move |event_arg, on_enter_exit_widget_rfn: (Function, Function)| {
                    main_return_data_clone.get_or_init(|| MainReturnData {
                        event_arg: Box::leak(Box::new(SafeValue(event_arg))),
                        on_enter_widget_rfn: on_enter_exit_widget_rfn
                            .0
                            .into_sync()
                            .into_result()
                            .expect("[ERROR] Failed turning on_enter_widget into a SyncFunction!"),
                        on_exit_widget_rfn: on_enter_exit_widget_rfn
                            .1
                            .into_sync()
                            .into_result()
                            .expect("[ERROR] Failed turning on_exit_widget into a SyncFunction!"),
                    });
                },
            )
            .build()
            .unwrap();
        context.install(STDExtCrate::build(Arc::clone(&self.system_utils), self_arc))?;
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

        vm.call(["main"], ())?;
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
            println!("[INFO] No on_ui_pre_init function, skipping.");
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

    /// Starts a new background loop.
    pub fn start_background_loop(
        &self,
        identifier: String,
        loop_function: SyncFunction,
        time: u64,
    ) {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[ERROR] No runtime config has been created, background loops cannot be started!"
            );
            return;
        };

        let event_arg = main_return_data.event_arg;
        gtk::glib::timeout_add(std::time::Duration::from_millis(time), move || {
            if let Err(error) = loop_function.call::<_, ()>((&event_arg.0,)).into_result() {
                eprintln!(
                    "[ERROR] Background loop \"{identifier}\" has panicked and been immediately stopped!"
                );
                eprintln!("[ERROR] Timeout: {time}, Error: {error}");
                return gtk::glib::ControlFlow::Break;
            }

            gtk::glib::ControlFlow::Continue
        });
    }
}
