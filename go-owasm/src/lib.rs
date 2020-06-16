mod env;
mod error;
mod span;
mod vm;

use env::Env;
use error::Error;
use parity_wasm::elements::{self};
use pwasm_utils::{self, rules};
use span::Span;
use std::ffi::c_void;
use wabt::wat2wasm;
use wasmer_runtime::{instantiate, Ctx};
use wasmer_runtime_core::error::RuntimeError;
use wasmer_runtime_core::{func, imports, wasmparser, Func};


#[no_mangle]
pub extern "C" fn do_compile(input: Span, output: &mut Span) -> Error {
    match compile(input.read()) {
        Ok(out) => {
            output.write(&out);
            Error::NoError
        },
        Err(e) => e,
    }
}

#[no_mangle]
pub extern "C" fn do_run(code: Span, gas_limit: u32, is_prepare: bool, env: Env) -> Error {
    match run(code.read(), gas_limit, is_prepare, env) {
        Ok(_) => Error::NoError,
        Err(e) => e,
    }
}

#[no_mangle]
pub extern "C" fn do_wat2wasm(input: Span, output: &mut Span) -> Error {
    match wat2wasm(input.read()) {
        Ok(_wasm) => output.write(&_wasm),
        Err(e) => match e.kind() {
            wabt::ErrorKind::Parse(_) => Error::ParseError,
            wabt::ErrorKind::WriteBinary => Error::WriteBinaryError,
            wabt::ErrorKind::ResolveNames(_) => Error::ResolveNamesError,
            wabt::ErrorKind::Validate(_) => Error::ValidateError,
            _ => Error::UnknownError,
        },
    }
}

fn compile(code: &[u8]) -> Result<Vec<u8>, Error> {
    // Check that the given Wasm code is indeed a valid Wasm.
    wasmparser::validate(code, None).map_err(|_| Error::ValidateError)?;
    // Simple gas rule. Every opcode and memory growth costs 1 gas.
    let gas_rules = rules::Set::new(1, Default::default()).with_grow_cost(1);
    // Start the compiling chains. TODO: Add more safeguards.
    let module = elements::deserialize_buffer(code).map_err(|_| Error::DeserializationError)?;
    let module = pwasm_utils::inject_gas_counter(module, &gas_rules).map_err(|_| Error::GasCounterInjectionError)?;
    // Serialize the final Wasm code back to bytes.
    elements::serialize(module).map_err(|_| Error::SerializationError)
}

struct ImportReference(*mut c_void);
unsafe impl Send for ImportReference {}
unsafe impl Sync for ImportReference {}

fn run(code: &[u8], gas_limit: u32, is_prepare: bool, env: Env) -> Result<(), Error> {
    let vm = &mut vm::VMLogic::new(env, gas_limit);
    let raw_ptr = vm as *mut _ as *mut c_void;
    let import_reference = ImportReference(raw_ptr);
    let import_object = imports! {
        move || (import_reference.0, (|_: *mut c_void| {}) as fn(*mut c_void)),
        "env" => {
            "gas" => func!(|ctx: &mut Ctx, gas: u32| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.consume_gas(gas)
            }),
            "get_calldata_size" => func!(|ctx: &mut Ctx| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_calldata().len as i64
            }),
            "read_calldata" => func!(|ctx: &mut Ctx, ptr: i64, len: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                let calldata = vm.get_calldata();
                for (byte, cell) in calldata.read().iter().zip(ctx.memory(0).view()[ptr as usize..(ptr + len) as usize].iter()) { cell.set(*byte); }
            }),
            "set_return_data" => func!(|ctx: &mut Ctx, ptr: i64, len: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                let data: Vec<u8> = ctx.memory(0).view()[ptr as usize..(ptr + len) as usize].iter().map(|cell| cell.get()).collect();
                vm.set_return_data(&data)
            }),
            "get_ask_count" => func!(|ctx: &mut Ctx| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_ask_count()
            }),
            "get_min_count" => func!(|ctx: &mut Ctx| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_min_count()
            }),
            "get_ans_count" => func!(|ctx: &mut Ctx| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_ans_count()
            }),
            "ask_external_data" => func!(|ctx: &mut Ctx, eid: i64, did: i64, ptr: i64, len: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                let data: Vec<u8> = ctx.memory(0).view()[ptr as usize..(ptr + len) as usize].iter().map(|cell| cell.get()).collect();
                vm.ask_external_data(eid, did, &data)
            }),
            "get_external_data_status" => func!(|ctx: &mut Ctx, eid: i64, vid: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_external_data_status(eid, vid)
            }),
            "get_external_data_size" => func!(|ctx: &mut Ctx, eid: i64, vid: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                vm.get_external_data(eid, vid).len as i64
            }),
            "read_external_data" => func!(|ctx: &mut Ctx, eid: i64, vid: i64, ptr: i64, len: i64| {
                let vm: &mut vm::VMLogic = unsafe { &mut *(ctx.data as *mut vm::VMLogic) };
                let calldata = vm.get_external_data(eid, vid);
                for (byte, cell) in calldata.read().iter().zip(ctx.memory(0).view()[ptr as usize..(ptr + len) as usize].iter()) { cell.set(*byte); }
            }),
        },
    };
    let instance = instantiate(code, &import_object).map_err(|_| Error::CompliationError)?;
    // TODO: remove this when we implement export safeguard
    let entry = if is_prepare { "prepare" } else { "execute" };
    let function: Func<(), ()> = instance
        .exports
        .get(entry)
        .map_err(|_| Error::FunctionNotFoundError)?;
    function.call().map_err(|err| match err {
        RuntimeError::User(uerr) => match uerr.downcast_ref::<vm::VmError>() {
            None => Error::UnknownError,
            Some(vm::VmError::GasLimitExceeded) => Error::GasLimitExceedError,
        },
        _ => Error::RunError,
    })
}
