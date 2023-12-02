mod model;
mod upgrade;

use candid::{candid_method, IDLArgs, IDLValue};
use ic_cdk::caller;
use ic_cdk_macros::{init, query, update};
use model::{Function, FunctionMapping, HeaderField, HttpRequest, HttpResponse};
use upgrade::STATE;

#[init]
fn init() {
    ic_cdk::setup();
    STATE.with(|s| {
        s.borrow_mut().owner = Some(caller());
    });
}

#[update]
#[candid_method(update)]
fn create_function(function: Function) -> bool {
    let mut result = true;
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.owner {
            Some(owner) => {
                if owner == caller() {
                    state
                        .function_types
                        .insert(function.func_name.clone(), function.clone());
                }
            }
            None => result = false,
        }
    });
    return result;
}

#[update]
#[candid_method(update)]
fn create_function_mapping(function_mapping: FunctionMapping) -> bool {
    let mut result = true;
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.owner {
            Some(owner) => {
                if owner == caller() {
                    state.function_mapping.insert(
                        function_mapping.request_func.func_name.clone(),
                        function_mapping.clone(),
                    );
                }else {
                     result = false
                }
            }
            None => result = false,
        }
    });
    let request_func = function_mapping.request_func;
    match create_function(request_func.clone()) {
        true => {
            ic_cdk::println!(
                "create_function_mapping insert function {} to memory is success",
                request_func.func_name
            )
        }
        false => {
            ic_cdk::println!(
                "create_function_mapping insert function {} to memory is fail",
                request_func.func_name
            );
            result = false
        }
    }
    let forward_func = function_mapping.forward_func;
    match create_function(forward_func.clone()) {
        true => {
            ic_cdk::println!(
                "create_function_mapping insert function {} to memory is success",
                forward_func.func_name
            )
        }
        false => {
            ic_cdk::println!(
                "create_function_mapping insert function {} to memory is fail",
                forward_func.func_name
            );
            result = false
        }
    }
    return result;
}

#[query]
#[candid_method(query)]
fn query_function(name: String) -> Vec<Function> {
    let mut ret = Vec::<Function>::new();
    STATE.with(|state| {
        let state = state.borrow();
        for (_key, _function_type) in state.function_types.iter() {
            if (name.is_empty()) || (!name.is_empty() && _function_type.func_name == name) {
                ret.push(_function_type.clone());
            }
        }
    });
    return ret;
}

#[query]
#[candid_method(query)]
fn query_function_mapping() -> Vec<FunctionMapping> {
    let mut ret = Vec::<FunctionMapping>::new();
    STATE.with(|state| {
        let state = state.borrow();
        for (_key, _function_mapping) in state.function_mapping.iter() {
            ret.push(_function_mapping.clone());
        }
    });
    return ret;
}

#[query]
#[candid_method(query)]
fn http_request(_request: HttpRequest) -> HttpResponse {
    let result = String::new();
    HttpResponse {
        status_code: 200,
        upgrade: true,
        headers: vec![
            HeaderField(
                "Content-Length".to_string(),
                format!("{}", result.as_bytes().len()),
            ),
            HeaderField("Cache-Control".to_string(), format!("max-age={}", 600)),
        ],
        body: result.as_bytes().to_vec(),
    }
}

#[update]
#[candid_method(update)]
async fn http_request_update(request: HttpRequest) -> HttpResponse {
    let mut _result = String::new();
    let request_func = "query_function_mapping";
    let forward_args_idl_str = "()";
    let request_canister = ic_cdk::id();

    let idl_args: IDLArgs = forward_args_idl_str
        .parse()
        .expect("Failed to parse arguments.");
    let forward_bytes = idl_args.to_bytes().expect("Failed to encode arguments.");

    match ic_cdk::api::call::call_raw(request_canister, &request_func, &forward_bytes, 0).await
    {
        Ok(raw_response) => {
            //get idl value from response
            let mut de = candid::de::IDLDeserialize::new(&raw_response).unwrap();
            let idl_value = de.get_value::<IDLValue>().unwrap();
            de.done().unwrap();
            ic_cdk::println!("idl value is {}", idl_value);
            _result = format!("{}", idl_value);
        }
        Err((r, m)) => {
            _result = format!("call canister error,code: {:#?} message: {}", r, m);
        }
    };
    return HttpResponse::build(_result.as_bytes().to_vec());
}
