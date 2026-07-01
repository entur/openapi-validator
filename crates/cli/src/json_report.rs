use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigValue {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Serialize)]
pub struct GeneratorList {
    pub server: Vec<String>,
    pub client: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<CustomGeneratorInfo>,
}

#[derive(Serialize)]
pub struct CustomGeneratorInfo {
    pub name: String,
    pub scope: String,
    pub has_compile: bool,
}
