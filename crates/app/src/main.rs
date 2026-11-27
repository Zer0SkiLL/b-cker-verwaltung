// Verhindert unter Windows ein zusätzliches Konsolenfenster im Release-Build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, RunEvent, WindowEvent};
use verwaltung_app::tauri_commands::{catalog, close_session, data, entities, session, AppState};

fn main() {
    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Info)
                .max_file_size(1_000_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            log::info!("Programmstart, Version {}", env!("CARGO_PKG_VERSION"));
            verwaltung_app::commands::resources::start_peak_sampler();
            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState::new(config_dir));
            #[cfg(debug_assertions)]
            open_db_for_tests(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                close_session(&window.state::<AppState>());
            }
        })
        .invoke_handler(tauri::generate_handler![
            session::last_path,
            session::open_file,
            session::create_file,
            session::take_over,
            session::session_info,
            session::close_file,
            session::diagnostics_log_tail,
            session::diagnostics_system,
            session::diagnostics_snapshot,
            data::search_addresses,
            data::count_active_members,
            data::export_addresses,
            data::list_groups,
            data::save_group,
            data::load_group,
            data::delete_group,
            data::rename_group,
            data::filter_options,
            data::backup_overview,
            data::set_secondary_dir,
            data::backup_now,
            data::restore_backup,
            data::export_all,
            data::import_preview,
            data::import_apply,
            data::import_undo,
            data::backup_if_changed,
            data::get_retention,
            data::set_retention,
            data::export_import_preview,
            data::export_report,
            entities::organisation_detail,
            entities::person_detail,
            entities::list_people,
            entities::list_organisations,
            entities::create_organisation,
            entities::update_organisation,
            entities::delete_organisation,
            entities::create_person,
            entities::update_person,
            entities::delete_person,
            entities::create_affiliation,
            entities::update_affiliation,
            entities::delete_affiliation,
            entities::create_membership,
            entities::update_membership,
            entities::delete_membership,
            entities::assign_label,
            entities::unassign_label,
            catalog::list_lookups,
            catalog::create_lookup,
            catalog::rename_lookup,
            catalog::delete_lookup,
            catalog::list_label_tree,
            catalog::create_label_group,
            catalog::create_label,
            catalog::rename_label,
            catalog::delete_label,
            catalog::delete_label_group,
            catalog::rename_label_group,
            catalog::list_teams,
            catalog::team_members,
            catalog::create_team,
            catalog::update_team,
            catalog::delete_team,
            catalog::add_team_member,
            catalog::remove_team_member,
        ])
        .build(tauri::generate_context!())
        .expect("Programm konnte nicht gestartet werden");
    // Auch bei Cmd+Q, Abmelden oder Herunterfahren sauber schliessen.
    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            close_session(&handle.state::<AppState>());
        }
    });
}

/// Debug-Builds: Datei aus `MITGLIEDERVERWALTUNG_DB` öffnen (für E2E-Tests ohne Dateidialog).
#[cfg(debug_assertions)]
fn open_db_for_tests(app: &tauri::App) {
    use verwaltung_app::startup::{db_from_env, open_startup_db, ENV_VAR};
    if let Some(path) = db_from_env(std::env::var(ENV_VAR).ok()) {
        let who = verwaltung_app::identity::identity_from_env();
        if let Err(err) = open_startup_db(&app.state::<AppState>(), &path, &who) {
            eprintln!("Testdatei konnte nicht geöffnet werden: {err}");
        }
    }
}
