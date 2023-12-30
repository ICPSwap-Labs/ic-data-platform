use candid::idl_hash;


pub(crate) fn field_name_to_hash_format(field_name: &String) -> String {
    let id_hash = field_name_to_hash(field_name);
    pp_num_str(id_hash.as_str())
}

fn field_name_to_hash(field_name: &String) -> String {
    let id_hash = idl_hash(field_name);
    return id_hash.to_string();
}

fn pp_num_str(s: &str) -> String {
    let mut groups = Vec::new();
    for chunk in s.as_bytes().rchunks(3) {
        let str = String::from_utf8_lossy(chunk);
        groups.push(str);
    }
    groups.reverse();
    if "-" == groups.first().unwrap() {
        "-".to_string() + &groups[1..].join("_")
    } else {
        groups.join("_")
    }
}