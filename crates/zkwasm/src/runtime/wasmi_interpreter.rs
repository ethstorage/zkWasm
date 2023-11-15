use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use specs::host_function::HostFunctionDesc;
use specs::jtable::StaticFrameEntry;
use specs::state::InitializationState;
use specs::CompilationTable;
use specs::ExecutionTable;
use specs::Tables;
use wasmi::Externals;
use wasmi::ImportResolver;
use wasmi::ModuleInstance;
use wasmi::RuntimeValue;
use wasmi::DEFAULT_VALUE_STACK_LIMIT;

// use super::state::UpdateCompilationTable;
use super::CompiledImage;
use super::ExecutionResult;

pub struct WasmRuntimeIO {
    pub public_inputs_and_outputs: Rc<RefCell<Vec<u64>>>,
    pub outputs: Rc<RefCell<Vec<u64>>>,
}

impl WasmRuntimeIO {
    pub fn empty() -> Self {
        Self {
            public_inputs_and_outputs: Rc::new(RefCell::new(vec![])),
            outputs: Rc::new(RefCell::new(vec![])),
        }
    }
}

pub trait Execution<R> {
    fn dry_run<E: Externals>(self, externals: &mut E) -> Result<Option<R>>;

    fn run<E: Externals>(
        self,
        externals: &mut E,
        wasm_io: WasmRuntimeIO,
    ) -> Result<ExecutionResult<R>>;
}

impl Execution<RuntimeValue>
    for CompiledImage<wasmi::NotStartedModuleRef<'_>, wasmi::tracer::Tracer>
{
    fn dry_run<E: Externals>(self, externals: &mut E) -> Result<Option<RuntimeValue>> {
        let instance = self.instance.run_start(externals).unwrap();

        let result = instance.invoke_export(&self.entry, &[], externals)?;

        Ok(result)
    }

    fn run<E: Externals>(
        self,
        externals: &mut E,
        wasm_io: WasmRuntimeIO,
    ) -> Result<ExecutionResult<RuntimeValue>> {
        let instance = self
            .instance
            .run_start_tracer(externals, self.tracer.clone())
            .unwrap();

        let result =
            instance.invoke_export_trace(&self.entry, &[], externals, self.tracer.clone())?;
        
        // because we've already write all the tables loader's callback
        // there is no need to write 

        let execution_tables = {
            let tracer = self.tracer.borrow();

            ExecutionTable {
                etable: tracer.etable.clone(),
                jtable: Arc::new(tracer.jtable.clone()),
            }
        };

        // let updated_init_memory_table = self
        //     .tables
        //     .update_init_memory_table(&execution_tables.etable);
        let updated_init_memory_table = self.tables.imtable.clone();

        let post_image_table = {
            CompilationTable {
                itable: self.tables.itable.clone(),
                imtable: updated_init_memory_table,
                elem_table: self.tables.elem_table.clone(),
                configure_table: self.tables.configure_table.clone(),
                static_jtable: self.tables.static_jtable.clone(),
                initialization_state: self
                    .tables.initialization_state.clone(),
                    // .update_initialization_state(&execution_tables.etable, true),
            }
        };

        Ok(ExecutionResult {
            tables: Tables {
                compilation_tables: self.tables,
                execution_tables,
                post_image_table,
            },
            result,
            public_inputs_and_outputs: wasm_io.public_inputs_and_outputs.borrow().clone(),
            outputs: wasm_io.outputs.borrow().clone(),
        })
    }
}

pub struct WasmiRuntime;

impl WasmiRuntime {
    pub fn new() -> Self {
        WasmiRuntime
    }

    pub fn compile<'a, I: ImportResolver>(
        module: &'a wasmi::Module,
        imports: &I,
        host_plugin_lookup: &HashMap<usize, HostFunctionDesc>,
        entry: &str,
        phantom_functions: &Vec<String>,
        callback: impl FnMut(wasmi::tracer::TracerCompilationTable) +'static
    ) -> Result<CompiledImage<wasmi::NotStartedModuleRef<'a>, wasmi::tracer::Tracer>> {
        let tracer = wasmi::tracer::Tracer::new(host_plugin_lookup.clone(), phantom_functions, callback);
        let tracer = Rc::new(RefCell::new(tracer));

        let instance = ModuleInstance::new(&module, imports, Some(tracer.clone()))
            .expect("failed to instantiate wasm module");

        let fid_of_entry = {
            let idx_of_entry = instance.lookup_function_by_name(tracer.clone(), entry);

            tracer
                .clone()
                .borrow_mut()
                .static_jtable_entries
                .push(StaticFrameEntry {
                    enable: true,
                    frame_id: 0,
                    next_frame_id: 0,
                    callee_fid: idx_of_entry,
                    fid: 0,
                    iid: 0,
                });

            if instance.has_start() {
                tracer
                    .clone()
                    .borrow_mut()
                    .static_jtable_entries
                    .push(StaticFrameEntry {
                        enable: true,
                        frame_id: 0,
                        next_frame_id: 0,
                        callee_fid: 0, // the fid of start function is always 0
                        fid: idx_of_entry,
                        iid: 0,
                    });
            }

            if instance.has_start() {
                0
            } else {
                idx_of_entry
            }
        };

        let itable = Arc::new(tracer.borrow().itable.clone());
        let imtable = tracer.borrow().imtable.finalized();
        let elem_table = Arc::new(tracer.borrow().elem_table.clone());
        let configure_table = Arc::new(tracer.borrow().configure_table.clone());
        let static_jtable = Arc::new(tracer.borrow().static_jtable_entries.clone());
        let initialization_state = InitializationState {
            eid: 1,
            fid: fid_of_entry,
            iid: 0,
            frame_id: 0,
            sp: DEFAULT_VALUE_STACK_LIMIT as u32 - 1,

            host_public_inputs: 1,
            context_in_index: 1,
            context_out_index: 1,
            external_host_call_call_index: 1,

            initial_memory_pages: configure_table.init_memory_pages,
            maximal_memory_pages: configure_table.maximal_memory_pages,

            #[cfg(feature = "continuation")]
            jops: 0,
        };

        Ok(CompiledImage {
            entry: entry.to_owned(),
            tables: CompilationTable {
                itable,
                imtable,
                elem_table,
                configure_table,
                static_jtable,
                initialization_state,
            },
            instance,
            tracer,
        })
    }
}
