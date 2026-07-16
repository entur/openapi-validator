use serde::Serialize;

use crate::custom::CustomGeneratorDef;

/// A built-in generator definition.
///
/// Serializable so frontends can expose the catalog directly (e.g. over IPC).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GeneratorDef {
    pub name: &'static str,
    pub scope: &'static str,
    pub config_yaml: &'static str,
}

// ── Server generators ────────────────────────────────────────────────

pub static SERVER_GENERATORS: &[GeneratorDef] = &[
    GeneratorDef {
        name: "spring",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/spring.yaml"),
    },
    GeneratorDef {
        name: "kotlin-spring",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/kotlin-spring.yaml"),
    },
    GeneratorDef {
        name: "go-server",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/go-server.yaml"),
    },
    GeneratorDef {
        name: "python-fastapi",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/python-fastapi.yaml"),
    },
    GeneratorDef {
        name: "aspnetcore",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/aspnetcore.yaml"),
    },
    GeneratorDef {
        name: "typescript-nestjs",
        scope: "server",
        config_yaml: include_str!("../assets/generators/server/typescript-nestjs.yaml"),
    },
];

// ── Client generators ────────────────────────────────────────────────

pub static CLIENT_GENERATORS: &[GeneratorDef] = &[
    GeneratorDef {
        name: "java",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/java.yaml"),
    },
    GeneratorDef {
        name: "kotlin",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/kotlin.yaml"),
    },
    GeneratorDef {
        name: "python",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/python.yaml"),
    },
    GeneratorDef {
        name: "go",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/go.yaml"),
    },
    GeneratorDef {
        name: "csharp",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/csharp.yaml"),
    },
    GeneratorDef {
        name: "typescript-axios",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/typescript-axios.yaml"),
    },
    GeneratorDef {
        name: "typescript-fetch",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/typescript-fetch.yaml"),
    },
    GeneratorDef {
        name: "typescript-node",
        scope: "client",
        config_yaml: include_str!("../assets/generators/client/typescript-node.yaml"),
    },
];

// ── Public API ───────────────────────────────────────────────────────

pub fn builtin_server_generators() -> &'static [GeneratorDef] {
    SERVER_GENERATORS
}

pub fn builtin_client_generators() -> &'static [GeneratorDef] {
    CLIENT_GENERATORS
}

pub fn builtin_generators_for_scope(scope: &str) -> &'static [GeneratorDef] {
    match scope {
        "server" => SERVER_GENERATORS,
        "client" => CLIENT_GENERATORS,
        _ => &[],
    }
}

pub fn find_builtin(name: &str, scope: &str) -> Option<&'static GeneratorDef> {
    builtin_generators_for_scope(scope)
        .iter()
        .find(|g| g.name == name)
}

/// All built-in generators, server first, then client.
pub fn builtin_generators() -> impl Iterator<Item = &'static GeneratorDef> {
    SERVER_GENERATORS.iter().chain(CLIENT_GENERATORS.iter())
}

pub fn server_names() -> Vec<&'static str> {
    SERVER_GENERATORS.iter().map(|g| g.name).collect()
}

pub fn client_names() -> Vec<&'static str> {
    CLIENT_GENERATORS.iter().map(|g| g.name).collect()
}

/// Built-in plus custom server generator names.
pub fn all_server_names(custom: &[CustomGeneratorDef]) -> Vec<String> {
    let mut names: Vec<String> = SERVER_GENERATORS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    names.extend(crate::custom::server_names(custom));
    names
}

/// Built-in plus custom client generator names.
pub fn all_client_names(custom: &[CustomGeneratorDef]) -> Vec<String> {
    let mut names: Vec<String> = CLIENT_GENERATORS
        .iter()
        .map(|g| g.name.to_string())
        .collect();
    names.extend(crate::custom::client_names(custom));
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_generator_count() {
        assert_eq!(builtin_server_generators().len(), 6);
    }

    #[test]
    fn client_generator_count() {
        assert_eq!(builtin_client_generators().len(), 8);
    }

    #[test]
    fn find_builtin_spring() {
        let def = find_builtin("spring", "server").unwrap();
        assert_eq!(def.name, "spring");
        assert!(def.config_yaml.contains("generatorName: spring"));
    }

    #[test]
    fn find_builtin_typescript_axios() {
        let def = find_builtin("typescript-axios", "client").unwrap();
        assert_eq!(def.name, "typescript-axios");
        assert!(def.config_yaml.contains("generatorName: typescript-axios"));
    }

    #[test]
    fn find_builtin_unknown_returns_none() {
        assert!(find_builtin("nonexistent", "server").is_none());
    }

    #[test]
    fn find_builtin_wrong_scope_returns_none() {
        assert!(find_builtin("spring", "client").is_none());
    }

    #[test]
    fn all_configs_contain_oav_output_dir() {
        for def in SERVER_GENERATORS.iter().chain(CLIENT_GENERATORS.iter()) {
            assert!(
                def.config_yaml.contains("outputDir: .oav/generated/"),
                "config for {} missing .oav/generated/ outputDir",
                def.name
            );
        }
    }

    #[test]
    fn server_names_returns_all() {
        let names = server_names();
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"spring"));
        assert!(names.contains(&"kotlin-spring"));
    }

    #[test]
    fn client_names_returns_all() {
        let names = client_names();
        assert_eq!(names.len(), 8);
        assert!(names.contains(&"typescript-axios"));
        assert!(names.contains(&"java"));
    }

    #[test]
    fn builtin_generators_yields_all_scopes() {
        assert_eq!(builtin_generators().count(), 14);
    }

    #[test]
    fn generator_def_serializes_to_json() {
        let def = find_builtin("spring", "server").unwrap();
        let json = serde_json::to_value(def).unwrap();
        assert_eq!(json["name"], "spring");
        assert_eq!(json["scope"], "server");
        assert!(json["config_yaml"].as_str().unwrap().contains("spring"));
    }

    fn custom_def(name: &str, scope: &str) -> CustomGeneratorDef {
        CustomGeneratorDef {
            name: name.to_string(),
            scope: scope.to_string(),
            generate: crate::custom::GenerateBlock {
                image: "img".to_string(),
                command: "cmd".to_string(),
            },
            compile: None,
        }
    }

    #[test]
    fn all_server_names_includes_custom() {
        let custom = vec![
            custom_def("my-server", "server"),
            custom_def("my-client", "client"),
        ];
        let names = all_server_names(&custom);
        assert!(names.contains(&"spring".to_string()));
        assert!(names.contains(&"my-server".to_string()));
        assert!(!names.contains(&"my-client".to_string()));
    }

    #[test]
    fn all_client_names_includes_custom() {
        let custom = vec![custom_def("my-client", "client")];
        let names = all_client_names(&custom);
        assert!(names.contains(&"java".to_string()));
        assert!(names.contains(&"my-client".to_string()));
    }
}
