pub const SERVER_GENERATOR_NAMES: &[&str] = &[
    "aspnetcore",
    "go-server",
    "kotlin-spring",
    "python-fastapi",
    "spring",
    "typescript-nestjs",
];

pub const CLIENT_GENERATOR_NAMES: &[&str] = &[
    "csharp",
    "go",
    "java",
    "kotlin",
    "python",
    "typescript-axios",
    "typescript-fetch",
    "typescript-node",
];

pub fn names_for_scope(scope: &str) -> &'static [&'static str] {
    match scope {
        "server" => SERVER_GENERATOR_NAMES,
        "client" => CLIENT_GENERATOR_NAMES,
        _ => &[],
    }
}
