use rune::{
    runtime::{Function, Value},
    termcolor::{ColorChoice, StandardStream},
    Context, Diagnostics, Module, Source, Sources, Vm,
};
use std::{
    cell::{OnceCell, RefCell},
    sync::Arc,
};

/// Data collected from the `main` function return data.
struct MainReturnData {
    /// `on_tick` Rune function.
    pub on_tick_rfn: Function,

    /// Single argument to be passed into every event function, such as a structure instance.
    pub event_arg: Value,

    /// `on_button_click` Rune function.
    pub on_button_click_rfn: Function,

    /// `on_enter_widget` Rune function.
    pub on_enter_widget_rfn: Function,

    /// `on_exit_widget` Rune function.
    pub on_exit_widget_rfn: Function,
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
}

unsafe impl Send for ScriptEngine {}
unsafe impl Sync for ScriptEngine {}

impl ScriptEngine {
    /// Builds a new virtual machine from the source `input` and then calls the `main` function on the
    /// source.
    pub fn run_from_input(&self, input: &str) -> rune::support::Result<()> {
        let mut context = Context::with_default_modules()?;
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

        if let Some((
            on_tick_rfn,
            event_arg,
            on_button_click_rfn,
            on_enter_widget_rfn,
            on_exit_widget_rfn,
        )) = rune::from_value::<Option<(Function, Value, Function, Function, Function)>>(result)?
        {
            self.main_return_data.get_or_init(|| MainReturnData {
                on_tick_rfn,
                event_arg,
                on_button_click_rfn,
                on_enter_widget_rfn,
                on_exit_widget_rfn,
            });
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

    /// Gets the reference to `self.rune_vm`.
    pub fn get_vm(&self) -> &RefCell<Vm> {
        self.rune_vm
            .get()
            .expect("[ERROR] No Rune Virtual Machine has been stored!")
    }

    pub fn assign_ui_modules(&self, modules: Vec<Module>) {
        self.ui_modules.get_or_init(|| modules);
    }

    /// Calls the `on_tick` function on the active VM if present.
    pub fn call_tick(&self) -> bool {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[WARN] No event functions were stored at startup, skipping advanced events!"
            );
            return false;
        };

        main_return_data
            .on_tick_rfn
            .call::<_, ()>((&main_return_data.event_arg,))
            .into_result()
            .is_ok()
    }

    /// Calls the `on_enter_widget` or `on_exit_widget` function on the VM if present.
    pub fn call_enter_exit(&self, identifier: String, entered: bool) -> bool {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[WARN] No event functions were stored at startup, skipping advanced events!"
            );
            return false;
        };

        if entered {
            return main_return_data
                .on_enter_widget_rfn
                .call::<_, ()>((&main_return_data.event_arg, identifier))
                .into_result()
                .is_ok();
        }

        main_return_data
            .on_exit_widget_rfn
            .call::<_, ()>((&main_return_data.event_arg, identifier))
            .into_result()
            .is_ok()
    }

    /// Calls the `on_tick` function on the active VM if present.
    pub fn call_on_button_click(&self, identifier: String) {
        let Some(main_return_data) = self.main_return_data.get() else {
            eprintln!(
                "[WARN] No event functions were stored at startup, skipping advanced events!"
            );
            return;
        };

        main_return_data
            .on_button_click_rfn
            .call::<_, ()>((&main_return_data.event_arg, identifier))
            .into_result()
            .expect("[ERROR] VM failed to call on_button_click!");
    }

    /// Gets the tick-rate in milliseconds for how often `on_tick` should be called.
    /// Default value is **100**ms.
    pub fn get_tick_rate(&self) -> u64 {
        let Ok(vm) = self.get_vm().try_borrow() else {
            eprintln!("[ERROR] VM is already being borrowed, cannot borrow for now. Using 100ms as the tick-rate!");
            return 100;
        };

        let Ok(function) = vm.lookup_function(["get_tick_rate"]) else {
            eprintln!("[ERROR] VM has no get_tick_rate (u64) function, or it's not visible. Using 100ms as the tick-rate!");
            return 100;
        };

        rune::from_value(
            function
                .call(())
                .expect("[ERROR] VM failed to call get_tick_rate!"),
        )
        .expect("[ERROR] VM failed reading get_tick_rate return value as u64!")
    }
}
