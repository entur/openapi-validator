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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_value_serializes_key_and_value() {
        let cv = ConfigValue {
            key: "mode".to_string(),
            value: serde_json::json!("client"),
        };
        let json = serde_json::to_value(&cv).unwrap();
        assert_eq!(json, serde_json::json!({"key": "mode", "value": "client"}));
    }

    #[test]
    fn config_value_supports_non_string_values() {
        let cv = ConfigValue {
            key: "search_depth".to_string(),
            value: serde_json::json!(3),
        };
        let json = serde_json::to_value(&cv).unwrap();
        assert_eq!(json["value"], serde_json::json!(3));
    }

    #[test]
    fn generator_list_omits_empty_custom_field() {
        let list = GeneratorList {
            server: vec!["spring".to_string()],
            client: vec!["typescript-axios".to_string()],
            custom: vec![],
        };
        let json = serde_json::to_value(&list).unwrap();
        assert!(
            json.get("custom").is_none(),
            "empty custom should be omitted"
        );
        assert_eq!(json["server"], serde_json::json!(["spring"]));
        assert_eq!(json["client"], serde_json::json!(["typescript-axios"]));
    }

    #[test]
    fn generator_list_includes_populated_custom_field() {
        let list = GeneratorList {
            server: vec![],
            client: vec![],
            custom: vec![CustomGeneratorInfo {
                name: "my-gen".to_string(),
                scope: "server".to_string(),
                has_compile: true,
            }],
        };
        let json = serde_json::to_value(&list).unwrap();
        assert_eq!(
            json["custom"],
            serde_json::json!([{"name": "my-gen", "scope": "server", "has_compile": true}])
        );
    }

    #[test]
    fn custom_generator_info_serializes_all_fields() {
        let info = CustomGeneratorInfo {
            name: "my-gen".to_string(),
            scope: "client".to_string(),
            has_compile: false,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"name": "my-gen", "scope": "client", "has_compile": false})
        );
    }
}
