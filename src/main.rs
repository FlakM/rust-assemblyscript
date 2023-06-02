use anyhow::anyhow;
use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, RwLock},
};

use wasmer::{
    imports, AsStoreRef, Function, FunctionEnv, FunctionEnvMut, Instance, Memory, MemoryView,
    Module, RuntimeError, Store, Value, WasmPtr,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut store = Store::default();

    println!("Compiling module...");

    let module = Module::new(&store, include_bytes!("../transformation/build/debug.wasm"))?;

    let host_env = HostEnv::default();

    let env = FunctionEnv::new(&mut store, host_env.clone());
    let import_object = imports! {
        "env" => {
            "abort" => Function::new_typed(&mut store, abort)
        },
        "index" => {
            "log" => Function::new_typed_with_env(&mut store, &env , log),
        }
    };

    let instance = Instance::new(&mut store, &module, &import_object)?;

    println!("Creating the imported function...");

    {
        let mut ctx = env.as_mut(&mut store).write().unwrap();
        let memory = instance.exports.get_memory("memory")?;
        ctx.set_memory(memory.clone());

        let fn_pin = instance.exports.get_function("__pin")?;
        ctx.set_fn_pin(fn_pin.clone());

        let fn_new = instance.exports.get_function("__new")?;
        ctx.set_fn_new(fn_new.clone());
    }

    let test_json = r#"{"name":"John", "age":30}"#;
    //let test_json = r#"{}"#;

    let mut my_response = MockHttpResponse {
        status: 200,
        body: test_json.as_bytes().to_vec(),
    };

    println!("[0] my_response: {}", my_response);

    let transform_func = instance.exports.get_function("transform")?;
    let ptr = alloc_string(host_env.clone(), &mut store, test_json)?;
    let result = transform_func.call(&mut store, &[Value::I32(ptr)])?;

    let ptr = result
        .get(0)
        .ok_or(anyhow!("can't get new string pointer"))?
        .i32()
        .ok_or(anyhow!("can't get new string pointer"))?;

    let ctx = host_env.read().unwrap();
    let result = read_string_wasm(ctx.memory_view(&store), ptr)
        .map_err(|e| RuntimeError::new(e.to_string()))?;

    my_response.body = result.as_bytes().to_vec();

    println!("[1] my_response: {}", my_response);

    Ok(())
}

#[derive(Clone, Default)]
pub struct Env {
    memory: Option<Memory>,
    pub fn_new: Option<Function>,
    pub fn_pin: Option<Function>,
    pub fn_unpin: Option<Function>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            memory: None,
            fn_new: None,
            fn_pin: None,
            fn_unpin: None,
        }
    }

    /// Copy the lazy reference so that when it's initialized during the
    /// export phase, all the other references get a copy of it
    pub fn memory_clone(&self) -> Option<Memory> {
        self.memory.clone()
    }

    /// Set the memory of the WasiEnv (can only be done once)
    pub fn set_memory(&mut self, memory: Memory) {
        if self.memory.is_some() {
            panic!("Memory of a Env can only be set once!");
        }
        self.memory = Some(memory);
    }

    /// Providers safe access to the memory
    /// (it must be initialized before it can be used)
    pub fn memory_view<'a>(&'a self, store: &'a impl AsStoreRef) -> MemoryView<'a> {
        self.memory().view(store)
    }

    /// Get memory, that needs to have been set fist
    pub fn memory(&self) -> &Memory {
        self.memory.as_ref().expect("can't get memory")
    }

    pub fn fn_new(&self) -> &Function {
        self.fn_new.as_ref().expect("can't get function")
    }

    pub fn fn_pin(&self) -> &Function {
        self.fn_pin.as_ref().expect("can't get function")
    }

    pub fn set_fn_new(&mut self, fn_new: Function) {
        if self.fn_new.is_some() {
            panic!("fn_new of a Env can only be set once!");
        }
        self.fn_new = Some(fn_new);
    }

    pub fn set_fn_pin(&mut self, fn_pin: Function) {
        if self.fn_pin.is_some() {
            panic!("fn_pin of a Env can only be set once!");
        }
        self.fn_pin = Some(fn_pin);
    }
}

fn read_string_wasm(view: MemoryView, string_ptr: i32) -> anyhow::Result<String> {
    let ptr: WasmPtr<u16> = WasmPtr::new(string_ptr as _);
    let offset = ptr.offset() / 4 - 1;
    // in assemblyscript, string's offset is -4
    // ptr / 4 - 1 if for u32, for u8, it need to be expanded 4 times
    let size = view.read_u8(offset as u64 * 4)?;
    // u8 -> u16, need / 2
    let values = ptr.slice(&view, size as u32 / 2)?;
    let values_sliced = values.read_to_vec().expect("qaq");
    let result = String::from_utf16_lossy(values_sliced.as_slice());

    Ok(result)
}

fn lift_string(ctx: &FunctionEnvMut<'_, HostEnv>, string_ptr: i32) -> anyhow::Result<String> {
    let env = ctx.data().read().unwrap();
    let view = env.memory_view(&ctx);
    read_string_wasm(view, string_ptr)
}

type HostEnv = Arc<RwLock<Env>>;

fn log(ctx: FunctionEnvMut<'_, HostEnv>, string_ptr: i32) -> Result<(), RuntimeError> {
    let result = lift_string(&ctx, string_ptr).map_err(|e| RuntimeError::new(e.to_string()))?;

    println!("{:#}", result);

    Ok(())
}

#[derive(Debug, Clone)]
struct MockHttpResponse {
    status: u32,
    body: Vec<u8>,
}

impl Display for MockHttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = String::from_utf8_lossy(&self.body);
        write!(f, "status: {}, body: {}", self.status, body)
    }
}

fn alloc_string(env: HostEnv, store: &mut Store, value: &str) -> anyhow::Result<i32> {
    let str_size = value.len() as i32;
    let env = env.read().unwrap();

    // todo handle unwrap
    let result = env
        .fn_new()
        .call(store, &[Value::I32(str_size << 1), Value::I32(1)])?;
    let ptr = result
        .get(0)
        .ok_or(anyhow!("can't get new string pointer"))?
        .i32()
        .ok_or(anyhow!("can't get new string pointer"))?;

    let utf16: Vec<u16> = value.encode_utf16().collect();
    let utf16_to_u8: &[u8] = bytemuck::try_cast_slice(&utf16.as_slice()).expect("qaq");

    let view = env.memory_view(store);

    view.write(ptr as u64, utf16_to_u8)?;
    env.fn_pin().call(store, &[Value::I32(ptr)])?;
    Ok(ptr)
}

fn abort(_msg: i32, _file: i32, _line: i32, _col: i32) {
    panic!("abort called");
}
