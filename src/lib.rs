mod model;
mod request_parser;
mod response_parser;
mod upgrade;
mod util;

use candid::{candid_method, IDLArgs, IDLValue, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{init, query, update};
use idl2json::{idl2json, Idl2JsonOptions};
use serde_json::Value;

use model::{Function, FunctionMapping, HeaderField, HttpRequest, HttpResponse, Log};
use request_parser::build_request_idl_str;
use response_parser::build_response_json_str;
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
fn create_function_mapping(function_mapping: FunctionMapping) -> Result<bool, String> {
    let mut result = true;
    let mut error = String::new();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.owner {
            Some(owner) => {
                if owner != caller() {
                    result = false;
                    error = "Only the owner has executive permission.".to_string();
                } else {
                    state.function_mapping.insert(
                        function_mapping.request_func.func_name.clone(),
                        function_mapping.clone(),
                    );
                }
            }
            None => {
                result = false;
                error = "No executive permission.".to_string();
            }
        }
    });

    if !result {
        insert_log(caller(), &error);
        return Err(error.to_string());
    }else {
        insert_log(caller(), "Access");
    }

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
            result = false;
            error.push_str("insert function ");
            error.push_str(&request_func.func_name);
            error.push_str(" to memory is fail");
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
            result = false;
            error.push_str("insert function ");
            error.push_str(&forward_func.func_name);
            error.push_str(" to memory is fail");
        }
    }
    if !result {
        return Err(error);
    }else {
        return Ok(result);
    }
}

fn insert_log(caller: Principal, log: &str) -> String {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.logs.push({
            Log {
                caller,
                content: log.to_string(),
                timestamp: ic_cdk::api::time(),
            }
        });
        if state.logs.len() > 2000 {
            state.logs.remove(0);
        }
    });
    log.to_string()
}

fn create_function(function: Function) -> bool {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.owner {
            Some(owner) => {
                assert!(
                    owner == caller(),
                    "Only the owner has executive permission."
                );
                state
                    .function_types
                    .insert(function.func_name.clone(), function.clone());
            }
            None => {
                assert!(false, "No executive permission");
            }
        }
    });
    true
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
fn query_log() -> Vec<Log> {
    return STATE.with(|state| {
        let state = state.borrow();
        return state.logs.clone();
    });
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
    let mut request_func = request.path().to_string();
    request_func.retain(|c| c != '/');
    if request_func.is_empty() {
        return HttpResponse::build("function name is empty".as_bytes().to_vec());
    }
    ic_cdk::println!("request function is {}", request_func);

    //request_json_object must json value
    let request_body: Value = match serde_json::from_slice(&request.body) {
        Ok(request_json_object) => request_json_object,
        Err(_error) => Value::String("{}".to_string()),
    };
    ic_cdk::println!("request data object is {}", request_body);

    //get function mapping from memory
    let function_mapping = STATE.with(|state| {
        let state = state.borrow();
        state.function_mapping.clone()
    });
    //get function type from memory
    let function_types = STATE.with(|state| {
        let state = state.borrow();
        state.function_types.clone()
    });

    let function_mapping = match function_mapping.get(&String::from(&request_func)) {
        Some(function_mapping) => function_mapping,
        None => {
            return HttpResponse::build("function mapping not found".as_bytes().to_vec());
        }
    };
    //build forward function args idl str
    let forward_args_idl_str =
        build_request_idl_str(request_body, function_mapping.request_mapping.clone());
    ic_cdk::println!("forward function args is {}", forward_args_idl_str);

    //get forward canister and function
    let forward_function_type = match function_types.get(&function_mapping.forward_func.func_name) {
        Some(forward_function_type) => forward_function_type,
        None => {
            return HttpResponse::build("forward function not found".as_bytes().to_vec());
        }
    };
    let forward_canister = forward_function_type.canister_id.unwrap();
    let forward_function = &forward_function_type.func_name;

    ic_cdk::println!("forward canister is {}", forward_canister);
    ic_cdk::println!("forward function is {}", forward_function);

    let idl_args: IDLArgs = forward_args_idl_str
        .parse()
        .expect("Failed to parse arguments.");
    let forward_bytes = idl_args.to_bytes().expect("Failed to encode arguments.");

    match ic_cdk::api::call::call_raw(forward_canister, &forward_function, &forward_bytes, 0).await
    {
        Ok(raw_response) => {
            //get idl value from response
            let mut de = candid::de::IDLDeserialize::new(&raw_response).unwrap();
            let idl_value = de.get_value::<IDLValue>().unwrap();
            de.done().unwrap();
            ic_cdk::println!("idl value is {}", idl_value);

            //transfer idl value to json value
            let value = &idl2json(&idl_value, &Idl2JsonOptions::default());
            ic_cdk::println!("json value is {}", value);

            _result =
                build_response_json_str(value.clone(), function_mapping.response_mapping.clone());
            ic_cdk::println!("result is {}", _result);
        }
        Err((r, m)) => {
            _result = format!("call canister error,code: {:#?} message: {}", r, m);
        }
    };
    return HttpResponse::build(_result.as_bytes().to_vec());
}
