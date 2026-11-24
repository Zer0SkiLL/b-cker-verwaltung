// Verhindert unter Windows ein zusätzliches Konsolenfenster im Release-Build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, RunEvent, WindowEvent};
use verwaltung_app::tauri_commands::{catalog, close_session, data, entities, session, AppState};

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState::new(config_dir));
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
            data::search_addresses,
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
