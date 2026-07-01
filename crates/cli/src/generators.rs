pub use oav_lib::generators::{CLIENT_GENERATORS, SERVER_GENERATORS, client_names, server_names};

pub fn all_server_names(custom: &[crate::custom::CustomGeneratorDef]) -> Vec<String> {
    let mut names: Vec<String> = SERVER_GENERATORS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    names.extend(crate::custom::server_names(custom));
    names
}

pub fn all_client_names(custom: &[crate::custom::CustomGeneratorDef]) -> Vec<String> {
    let mut names: Vec<String> = CLIENT_GENERATORS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    names.extend(crate::custom::client_names(custom));
    names
}
