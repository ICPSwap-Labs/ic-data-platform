use crate::model::{Entity, EntityMapping};
use serde_json::Value;

pub(crate) fn build_request_idl_str(
    request_value: Value,
    request_entity_mapping: EntityMapping,
) -> String {
    let mut idl_str = String::from("(");
    if request_value.is_object() {
        let request_value = request_value.clone();

        let source_entity = request_entity_mapping.source.clone();
        let target_entity = request_entity_mapping.target.clone();

        //transfer to tuple type values from Map<String,value>
        if target_entity.entity_type == "tuple" {
            ic_cdk::println!("is tuple");
            let mut index = 0;
            for target_field in target_entity.fields {
                match source_entity.fields.get(index) {
                    Some(source_field) => {
                        idl_str = build_general_idl_str(
                            request_value.clone(),
                            &source_field,
                            &target_field,
                            &mut idl_str,
                        );
                    }
                    None => {}
                }
                index += 1;
            }
        } else if target_entity.entity_type == "object" {
            ic_cdk::println!("is object");
            ic_cdk::println!(
                "object source field {},target field {},value {}",
                &source_entity.code,
                &target_entity.code,
                &request_value
            );
            let mut index = 0;
            for target_field in &target_entity.fields {
                match source_entity.fields.get(index) {
                    Some(source_field) => {
                        idl_str = build_object_idl_str(
                            request_value.clone(),
                            &source_field,
                            &target_field,
                            &mut idl_str,
                        );
                    }
                    None => {}
                }
                index += 1;
            }
        } else if target_entity.entity_type == "enum" {
            ic_cdk::println!("is enum");
            idl_str =
                build_enum_idl_str(request_value, &source_entity, &target_entity, &mut idl_str);
        } else {
            idl_str =
                build_general_idl_str(request_value, &source_entity, &target_entity, &mut idl_str);
        }
    } else {
        ic_cdk::println!("request data is not object,please check it.");
    }
    idl_str += ")";
    return idl_str;
}

fn build_enum_idl_str(
    request_value: Value,
    source: &Entity,
    target: &Entity,
    mut idl_str: &mut String,
) -> String {
    match idl_str.ends_with("(") || idl_str.ends_with("=") {
        true => (),
        false => {
            idl_str.push_str(",");
        }
    }

    match target.is_opt == true {
        true => match !request_value.is_null() {
            true => {
                idl_str.push_str("opt ");
            }
            false => (),
        },
        false => (),
    }

    idl_str.push_str("variant {");

    let mut index = 0;
    for target_field in &target.fields {
        match source.fields.get(index) {
            Some(source_field) => match request_value.get(&source_field.code) {
                Some(field_value) => match field_value.is_null() {
                    false => {
                        *idl_str = build_record_field_idl_str(
                            field_value.clone(),
                            &source_field,
                            &target_field,
                            &mut idl_str,
                        );
                    }
                    true => {}
                },
                None => {}
            },
            None => {}
        }
        index += 1;
    }
    idl_str.push_str("}");
    return idl_str.clone();
}

fn build_object_idl_str(
    request_value: Value,
    source: &Entity,
    target: &Entity,
    mut idl_str: &mut String,
) -> String {
    match idl_str.ends_with("(") || idl_str.ends_with("=") {
        true => (),
        false => {
            idl_str.push_str(",");
        }
    }

    if target.is_opt == true {
        if !request_value.is_null() {
            idl_str.push_str("opt ");
        }
    }

    idl_str.push_str("record {");

    let mut index = 0;
    for target_field in &target.fields {
        match source.fields.get(index) {
            Some(source_field) => match request_value.get(&source_field.code) {
                Some(field_value) => match target_field.entity_type == "enum" {
                    true => {
                        *idl_str = build_enum_idl_str(
                            field_value.clone(),
                            &source_field,
                            &target_field,
                            &mut idl_str,
                        );
                    }
                    false => {
                        match target_field.is_opt == true {
                            true => match field_value.is_null() {
                                false => {
                                    idl_str.push_str("opt ");
                                }
                                true => {}
                            },
                            false => {}
                        }
                        *idl_str = build_record_field_idl_str(
                            field_value.clone(),
                            &source_field,
                            &target_field,
                            &mut idl_str,
                        );
                    }
                },
                None => {}
            },
            None => {}
        }

        index += 1;
    }
    idl_str.push_str("}");
    return idl_str.clone();
}

fn build_record_field_idl_str(
    request_value: Value,
    source: &Entity,
    target: &Entity,
    mut idl_str: &mut String,
) -> String {
    match idl_str.ends_with("record {") || idl_str.ends_with("variant {") {
        true => (),
        false => {
            idl_str.push_str(";");
        }
    }
    idl_str.push_str(&target.code);
    idl_str.push_str("=");

    match target.entity_type == "principal" {
        true => {
            idl_str.push_str("principal ");
        }
        false => (),
    }

    if request_value.is_object() {
        ic_cdk::println!("record field is object,target field {}", &target.code);
        *idl_str = build_object_idl_str(request_value, &source, &target, &mut idl_str);
    } else if request_value.is_array() {
        ic_cdk::println!("record field is array,target field {}", &target.code);
        match request_value.as_array() {
            Some(request_array) => {
                idl_str.push_str("vec {");
                for ele in request_array {
                    match ele.is_object() {
                        true => {
                            ic_cdk::println!(
                                "record field is object array,target field {}",
                                &target.code
                            );
                            *idl_str =
                                build_object_idl_str(ele.clone(), &source, &target, &mut idl_str);
                        }
                        false => {
                            ic_cdk::println!(
                                "record field is other type array,target field {}",
                                &target.code
                            );
                            *idl_str =
                                build_general_idl_str(ele.clone(), &source, &target, &mut idl_str);
                        }
                    }
                    idl_str.push_str(";");
                }
                idl_str.push_str("}");
            }
            None => {}
        }
    } else if request_value.is_string() {
        ic_cdk::println!("record field is string,target field {}", &target.code);
        match request_value.as_str() {
            Some(request_str) => {
                idl_str.push_str("\"");
                idl_str.push_str(request_str);
                idl_str.push_str("\"");
            }
            None => {}
        }
    } else if request_value.is_number() {
        ic_cdk::println!("record field is number target field {}", &target.code);
        match request_value.as_number() {
            Some(request_number)=>{
                idl_str.push_str(&request_number.to_string());
            }
            None=>{}
        }
    } else if request_value.is_boolean() {
        ic_cdk::println!("record field is bool target field {}", &target.code);
        match request_value.as_bool() {
            Some(request_bool)=>{
                idl_str.push_str(&request_bool.to_string());
            }
            None=>{}
        }
    } else if request_value.is_null() {
        ic_cdk::println!("record field is null target field {}", &target.code);
        idl_str.push_str("null");
    }
    return idl_str.clone();
}

fn build_general_idl_str(
    request_value: Value,
    source: &Entity,
    target: &Entity,
    idl_str: &mut String,
) -> String {
    match idl_str.ends_with("(") || idl_str.ends_with("{") || idl_str.ends_with("=") {
        true => (),
        false => {
            idl_str.push_str(",");
        }
    }
    if target.is_opt == true {
        if !request_value.is_null() {
            idl_str.push_str("opt ");
        }
    }

    if target.entity_type == "principal" {
        idl_str.push_str("principal ");
    }
    match request_value.get(&source.code) {
        Some(element_value) => {
            if element_value.is_string() {
                ic_cdk::println!("element is string");
                match element_value.as_str() {
                    Some(request_str)=>{
                        idl_str.push_str("\"");
                idl_str.push_str(request_str);
                idl_str.push_str("\"");
                    }
                    None=>{}
                }
            } else if element_value.is_number() {
                ic_cdk::println!("element is number");
                match element_value.as_number() {
                    Some(request_number)=>{
                        idl_str.push_str(&request_number.to_string());
                    }
                    None=>{}
                }
            } else if element_value.is_boolean() {
                ic_cdk::println!("element is bool");
                match element_value.as_bool() {
                    Some(request_bool)=>{
                        idl_str.push_str(&request_bool.to_string());
                    }
                    None=>{}
                }
            } else if element_value.is_null() {
                ic_cdk::println!("element is null");
                idl_str.push_str("null");
            }
        }
        None => {
            idl_str.push_str("null");
            ic_cdk::println!("element is not found,source field is {}", source.code);
        }
    }
    return idl_str.clone();
}
