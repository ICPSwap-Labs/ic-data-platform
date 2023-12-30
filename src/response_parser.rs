use crate::util::field_name_to_hash_format;
use crate::model::{Entity, EntityMapping};
use serde_json::{json, Map, Value};


pub(crate) fn build_response_json_str(
    value_data: Value,
    response_entity_mapping: EntityMapping,
) -> String {
    let _source_entity = &response_entity_mapping.source;
    let _target_entity = &response_entity_mapping.target;

    if _source_entity.entity_type == "array" {
        match value_data.is_array() {
            true => {
                ic_cdk::println!("array value {}", value_data);
                let target_field_value =
                    get_array_value(&value_data, &_source_entity, &_target_entity);
                return serde_json::to_string(&target_field_value).unwrap();
            }
            false => {}
        }
    } else if _source_entity.entity_type == "object" {
        match value_data.is_object() {
            true => {
                let object_value = get_object_value(&value_data, &_source_entity, &_target_entity);
                return serde_json::to_string(&object_value).unwrap();
            }
            false => {}
        }
    } else if _source_entity.entity_type == "principal" {
        return get_principal_value(value_data.clone());
    } else {
        return serde_json::to_string(&value_data).unwrap();
    }
    let response_value = Map::new();
    return serde_json::to_string(&response_value).unwrap();
}

fn get_object_value(
    value_data: &Value,
    source_entity_type: &Entity,
    target_entity_type: &Entity,
) -> Value {
    let mut result: Map<String, Value> = Map::new();
    let mut index = 0;
    match value_data.as_object() {
        Some(value_object) => {
            ic_cdk::println!("object value ,{}", value_data);
            for target_field in target_entity_type.fields.iter() {
                match source_entity_type.fields.get(index) {
                    Some(source_field) => {
                        let value = get_value_by_field_code(value_object, &source_field.code);
                        if source_field.entity_type == "object" {
                            ic_cdk::println!("object value {}", value);
                            if value.is_object() {
                                let object_value =
                                    get_object_value(&value, &source_field, &target_field);
                                result.insert(target_field.code.clone(), object_value);
                            }
                        } else if source_field.entity_type == "array" {
                            ic_cdk::println!("array value {}", value);
                            if value.is_array() {
                                let array_value =
                                    get_array_value(&value, source_field, &target_field);
                                result.insert(target_field.code.to_string(), array_value);
                            }
                        } else if source_field.entity_type == "principal" {
                            let principal = get_principal_value(value.clone());
                            result.insert(target_field.code.clone(), Value::String(principal));
                        } else {
                            result.insert(target_field.code.clone(), value.clone());
                        }
                    }
                    None => {}
                }
                index += 1;
            }
        }
        None => {}
    }
    return Value::Object(result);
}

fn get_array_value(
    value_data: &Value,
    source_entity_type: &Entity,
    target_entity_type: &Entity,
) -> Value {
    ic_cdk::println!("array value = {}", value_data);
    let mut array_value: Vec<Value> = vec![];

    if value_data.is_array() {
        match value_data.as_array() {
            Some(value) => {
                for value_item in value.iter() {
                    if source_entity_type.fields.len() > 1 {
                        //child type is object/record when entity_type is array and fields len > 1
                        let record_value =
                            get_object_value(value_item, source_entity_type, target_entity_type);
                        array_value.push(record_value);
                    } else if source_entity_type.fields.len() == 1 {
                        if source_entity_type.fields[0].entity_type == "principal" {
                            let principal = get_principal_value(value_item.clone());
                            array_value.push(Value::String(principal));
                        } else {
                            array_value.push(value_item.clone());
                        }
                    } else {
                        ic_cdk::println!("source_entity_type.fields.len() == 0");
                    }
                }
            }
            None => return Value::Array(array_value),
        };
    }
    return Value::Array(array_value);
}

fn get_principal_value(value : Value) -> String{
    match value.as_array() {
        Some(principal_value) => match principal_value.get(0) {
            Some(principal_value_item) => {
                let string_value = principal_value_item.as_str();
                match string_value {
                    Some(string_value) => {
                        let mut value = string_value.to_string();
                        value.retain(|c| c != '[' || c != ']');
                        return value;
                    }
                    None => {}
                }
            }
            None => {}
        },
        None => {}
    }
    return String::new();
}

fn get_value_by_field_code(value_data: &Map<String, Value>, field_code: &String) -> Value {
    let empty_value = { json!({}) };
    ic_cdk::println!("get_value_by_field_code {}", field_code);
    let field_value = match value_data.get(field_code) {
        Some(field_value) => field_value,
        None => {
            let hash_id = &field_name_to_hash_format(field_code);
            ic_cdk::println!("get_value_by_field_code {}", hash_id);
            match value_data.get(hash_id) {
                Some(field_value) => field_value,
                None => &empty_value,
            }
        }
    };
    return field_value.clone();
}

