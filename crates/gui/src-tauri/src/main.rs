#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::PipelineState::default())
        .invoke_handler(tauri::generate_handler![
            commands::discover_specs,
            commands::load_config,
            commands::save_config,
            commands::validate_config,
            commands::generator_catalog,
            commands::docker_status,
            commands::read_workspace_file,
            commands::write_workspace_file,
            commands::list_generated_files,
            commands::parse_lint,
            commands::parse_compile,
            commands::load_report,
            commands::propose_fix,
            commands::apply_fix,
            commands::fetch_spec_url,
            commands::start_pipeline,
            commands::cancel_pipeline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
