use std::collections::BTreeMap;

use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};


#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Log {
    pub caller: Principal,
    pub content: String,
    pub timestamp: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entity {
    pub name: String,
    pub code: String,
    pub entity_type: String, //bool,number,string,principal,enum,array,object,tuple
    pub fields: Vec<Entity>,
    pub is_opt: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Function {
    pub func_name: String,
    pub func_type: String, //request,forward
    pub canister_id: Option<Principal>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntityMapping {
    pub source: Entity,
    pub target: Entity,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionMapping {
    pub request_func: Function,
    pub forward_func: Function,
    pub request_mapping: EntityMapping,
    pub response_mapping: EntityMapping,
}

impl ArgumentEncoder for Function {
    fn encode(self, ser: &mut candid::ser::IDLBuilder) -> candid::Result<()> {
        ser.arg(&self)?;
        Ok(())
    }
}

impl ArgumentDecoder<'_> for Function {
    fn decode<'a>(de: &mut candid::de::IDLDeserialize<'a>) -> candid::Result<Self> {
        let value: Function = de.get_value()?;
        Ok(value)
    }
}

impl ArgumentEncoder for FunctionMapping {
    fn encode(self, ser: &mut candid::ser::IDLBuilder) -> candid::Result<()> {
        ser.arg(&self)?;
        Ok(())
    }
}

impl ArgumentDecoder<'_> for FunctionMapping {
    fn decode<'a>(de: &mut candid::de::IDLDeserialize<'a>) -> candid::Result<Self> {
        let value: FunctionMapping = de.get_value()?;
        Ok(value)
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct Data {
    pub function_types: BTreeMap<String, Function>,
    pub function_mapping: BTreeMap<String, FunctionMapping>,
    pub owner: Option<Principal>,
    pub logs: Vec<Log>,
}

impl ArgumentEncoder for Data {
    fn encode(self, ser: &mut candid::ser::IDLBuilder) -> candid::Result<()> {
        ser.arg(&self)?;
        Ok(())
    }
}

impl ArgumentDecoder<'_> for Data {
    fn decode<'a>(de: &mut candid::de::IDLDeserialize<'a>) -> candid::Result<Self> {
        let value: Data = de.get_value()?;
        Ok(value)
    }
}

impl Default for Data {
    fn default() -> Self {
        Data {
            function_types: BTreeMap::new(),
            function_mapping: BTreeMap::new(),
            owner: None,
            logs: Vec::new(),
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn path(&self) -> &str {
        match self.url.find('?') {
            None => &self.url[..],
            Some(index) => &self.url[..index],
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
    pub upgrade: bool,
}

impl HttpResponse {
    pub fn build(response_body: Vec<u8>) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            upgrade: false,
            headers: vec![
                HeaderField(
                    "Content-Length".to_string(),
                    format!("{}", response_body.len()),
                ),
                HeaderField("Cache-Control".to_string(), format!("max-age={}", 600)),
            ],
            body: response_body,
        }
    }
}
