#![deny(unsafe_code)]
#![deny(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]

mod app;
mod converter;
mod frame;
mod hledger;
mod widgets;

use std::fs;

use tauri::{AppHandle, Manager};

use app::App;

use tauri_egui::egui::vec2;
use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

use crate::frame::state::app::State;

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .setup(|app| {
            let handle = app.handle().clone();
            init_logs(&handle);

            tracing::info!("starting app");

            app.wry_plugin(tauri_egui::EguiPluginBuilder::new(app.handle()));

            let state = State::try_from(&handle).unwrap_or_default();

            let native_options = tauri_egui::eframe::NativeOptions {
                initial_window_size: Some(vec2(state.window.size[0], state.window.size[1])),
                initial_window_pos: state.window.position.map(|p| p.into()),
                fullscreen: state.window.fullscreen,
                maximized: state.window.maximized,
                drag_and_drop_support: true,
                icon_data: None,
                #[cfg(target_os = "macos")]
                fullsize_content: true,
                ..Default::default()
            };

            let manager = hledger::Manager::from(&handle);
            app.manage(manager);

            app.state::<tauri_egui::EguiPluginHandle>()
                .create_window(
                    "main".to_string(),
                    Box::new(|cc| Box::new(App::new(cc, handle, state))),
                    "hledger-desktop".into(),
                    native_options,
                )
                .expect("failed to create window");

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {});
}

fn init_logs(handle: &AppHandle) {
    let logs_dir = handle
        .path()
        .app_log_dir()
        .expect("failed to get app log dir");
    fs::create_dir_all(&logs_dir).expect("failed to create logs dir");

    let file_appender = tracing_appender::rolling::never(&logs_dir, "log.txt");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    handle.manage(guard); // keep the guard alive for the lifetime of the app

    let log_format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact();

    let log_level = if cfg!(debug) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    set_global_default(
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(log_format.clone())
                    .with_span_events(FmtSpan::CLOSE)
                    .with_filter(log_level),
            )
            .with(
                // subscriber that writes spans to a file
                tracing_subscriber::fmt::layer()
                    .event_format(log_format)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_writer(file_writer)
                    .with_filter(log_level),
            ),
    )
    .expect("failed to set global logs subscriber");
}
