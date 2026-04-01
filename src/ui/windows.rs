//! Native Windows settings window (Win32 via native-windows-gui).

use anyhow::{Context, Result};
use native_windows_gui as nwg;

use crate::engine::config::{self, SaveTarget};
use crate::engine::settings::{self, SettingsDraft, SettingsValidationError};
use crate::i18n;

fn is_checked(cb: &nwg::CheckBox) -> bool {
    cb.check_state() == nwg::CheckBoxState::Checked
}

fn checked_state(v: bool) -> nwg::CheckBoxState {
    if v {
        nwg::CheckBoxState::Checked
    } else {
        nwg::CheckBoxState::Unchecked
    }
}

fn target_to_idx(target: SaveTarget) -> usize {
    match target {
        SaveTarget::NewFileInFolder => 0,
        SaveTarget::AppendToFile => 1,
    }
}

fn idx_to_target(idx: Option<usize>) -> SaveTarget {
    match idx {
        Some(1) => SaveTarget::AppendToFile,
        _ => SaveTarget::NewFileInFolder,
    }
}

fn language_to_idx(language: &str) -> usize {
    match language {
        "ru" => 1,
        "en" => 0,
        _ => match i18n::current() {
            i18n::Lang::Ru => 1,
            i18n::Lang::En => 0,
        },
    }
}

fn idx_to_language(idx: Option<usize>) -> String {
    match idx {
        Some(1) => "ru".to_string(),
        _ => "en".to_string(),
    }
}

fn show_validation_error(window: &nwg::Window, error: SettingsValidationError) {
    match error {
        SettingsValidationError::EmptySubfolder => {
            nwg::modal_info_message(window, i18n::alert_invalid(), i18n::alert_empty_subfolder());
        }
        SettingsValidationError::EmptyAppend => {
            nwg::modal_info_message(window, i18n::alert_invalid(), i18n::alert_empty_append());
        }
        SettingsValidationError::EmptyTemplate => {
            nwg::modal_info_message(window, i18n::alert_invalid(), i18n::alert_empty_template());
        }
        SettingsValidationError::InvalidHotkey(msg) => {
            nwg::modal_error_message(window, i18n::alert_invalid_hotkey(), &msg);
        }
        SettingsValidationError::InvalidPath(msg) => {
            nwg::modal_error_message(window, i18n::alert_invalid(), &msg);
        }
    }
}

pub fn open() -> Result<()> {
    nwg::init().context("initialize native-windows-gui")?;

    let mut ui_font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(14)
        .build(&mut ui_font)
        .context("create settings UI font")?;
    nwg::Font::set_global_default(Some(ui_font));

    let cfg = config::load_or_create().context("load config")?;
    i18n::apply(&cfg.language);

    let mut window = nwg::Window::default();
    nwg::Window::builder()
        .size((640, 620))
        .position((300, 200))
        .title(i18n::settings_title())
        .build(&mut window)
        .context("create settings window")?;

    let mut vault_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_vault())
        .position((12, 14))
        .size((260, 24))
        .parent(&window)
        .build(&mut vault_label)?;

    let mut vault_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .position((12, 38))
        .size((430, 28))
        .parent(&window)
        .text(&cfg.vault_path.to_string_lossy())
        .build(&mut vault_input)?;

    let mut browse_btn = nwg::Button::default();
    nwg::Button::builder()
        .text(i18n::settings_browse())
        .position((450, 38))
        .size((84, 28))
        .parent(&window)
        .build(&mut browse_btn)?;

    let mut detect_btn = nwg::Button::default();
    nwg::Button::builder()
        .text(i18n::settings_detect())
        .position((542, 38))
        .size((84, 28))
        .parent(&window)
        .build(&mut detect_btn)?;

    let mut mode_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_save_mode())
        .position((12, 76))
        .size((260, 24))
        .parent(&window)
        .build(&mut mode_label)?;

    let mut target_combo = nwg::ComboBox::<String>::default();
    nwg::ComboBox::builder()
        .position((12, 100))
        .size((614, 28))
        .parent(&window)
        .collection(vec![
            i18n::save_target_new_file().to_string(),
            i18n::save_target_append().to_string(),
        ])
        .build(&mut target_combo)?;
    target_combo.set_selection(Some(target_to_idx(cfg.target.clone())));

    let mut folder_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_folder())
        .position((12, 138))
        .size((260, 24))
        .parent(&window)
        .build(&mut folder_label)?;

    let mut folder_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .position((12, 162))
        .size((614, 28))
        .parent(&window)
        .text(&cfg.folder)
        .build(&mut folder_input)?;

    let mut append_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_append())
        .position((12, 200))
        .size((260, 24))
        .parent(&window)
        .build(&mut append_label)?;

    let mut append_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .position((12, 224))
        .size((614, 28))
        .parent(&window)
        .text(&cfg.append_file)
        .build(&mut append_input)?;

    let mut tpl_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_template())
        .position((12, 262))
        .size((300, 24))
        .parent(&window)
        .build(&mut tpl_label)?;

    let mut tpl_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .position((12, 286))
        .size((614, 28))
        .parent(&window)
        .text(&cfg.filename_template)
        .build(&mut tpl_input)?;

    let mut ts_check = nwg::CheckBox::default();
    nwg::CheckBox::builder()
        .text(i18n::settings_timestamp())
        .position((12, 322))
        .size((614, 24))
        .parent(&window)
        .build(&mut ts_check)?;
    ts_check.set_check_state(checked_state(cfg.prepend_timestamp_header));

    let mut autostart_check = nwg::CheckBox::default();
    nwg::CheckBox::builder()
        .text(i18n::settings_autostart())
        .position((12, 348))
        .size((614, 24))
        .parent(&window)
        .build(&mut autostart_check)?;
    autostart_check.set_check_state(checked_state(cfg.autostart));

    let mut auto_update_check = nwg::CheckBox::default();
    nwg::CheckBox::builder()
        .text(i18n::settings_auto_update())
        .position((12, 374))
        .size((614, 24))
        .parent(&window)
        .build(&mut auto_update_check)?;
    auto_update_check.set_check_state(checked_state(cfg.auto_update));

    let mut lang_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_lang())
        .position((12, 404))
        .size((260, 24))
        .parent(&window)
        .build(&mut lang_label)?;

    let mut lang_combo = nwg::ComboBox::<String>::default();
    nwg::ComboBox::builder()
        .position((12, 428))
        .size((614, 28))
        .parent(&window)
        .collection(vec!["English".to_string(), "Русский".to_string()])
        .build(&mut lang_combo)?;
    lang_combo.set_selection(Some(language_to_idx(&cfg.language)));

    let mut hotkey_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_hotkey())
        .position((12, 466))
        .size((300, 24))
        .parent(&window)
        .build(&mut hotkey_label)?;

    let mut hotkey_input = nwg::TextInput::default();
    nwg::TextInput::builder()
        .position((12, 490))
        .size((614, 28))
        .parent(&window)
        .text(cfg.hotkey.as_deref().unwrap_or(""))
        .build(&mut hotkey_input)?;

    let mut hint_label = nwg::Label::default();
    nwg::Label::builder()
        .text(i18n::settings_hotkey_hint())
        .position((12, 520))
        .size((614, 24))
        .parent(&window)
        .build(&mut hint_label)?;

    let mut open_raw_btn = nwg::Button::default();
    nwg::Button::builder()
        .text(i18n::settings_raw())
        .position((12, 550))
        .size((150, 32))
        .parent(&window)
        .build(&mut open_raw_btn)?;

    let mut cancel_btn = nwg::Button::default();
    nwg::Button::builder()
        .text(i18n::settings_cancel())
        .position((452, 550))
        .size((84, 32))
        .parent(&window)
        .build(&mut cancel_btn)?;

    let mut save_btn = nwg::Button::default();
    nwg::Button::builder()
        .text(i18n::settings_save())
        .position((542, 550))
        .size((84, 32))
        .parent(&window)
        .build(&mut save_btn)?;

    let mut dir_dialog = nwg::FileDialog::default();
    nwg::FileDialog::builder()
        .action(nwg::FileDialogAction::OpenDirectory)
        .title(i18n::alert_choose_vault())
        .build(&mut dir_dialog)?;

    let window_h = window.handle;
    let event_source = window.handle.clone();
    let save_h = save_btn.handle;
    let cancel_h = cancel_btn.handle;
    let raw_h = open_raw_btn.handle;
    let browse_h = browse_btn.handle;
    let detect_h = detect_btn.handle;

    let evt_handler =
        nwg::full_bind_event_handler(&event_source, move |evt, _evt_data, handle| match evt {
            nwg::Event::OnWindowClose if handle == window_h => nwg::stop_thread_dispatch(),
            nwg::Event::OnButtonClick if handle == cancel_h => nwg::stop_thread_dispatch(),
            nwg::Event::OnButtonClick if handle == raw_h => config::open_in_editor(),
            nwg::Event::OnButtonClick if handle == browse_h => {
                if dir_dialog.run(Some(&window)) {
                    if let Ok(path) = dir_dialog.get_selected_item() {
                        vault_input.set_text(&path.to_string_lossy());
                    }
                }
            }
            nwg::Event::OnButtonClick if handle == detect_h => {
                if let Some(path) = config::best_vault() {
                    vault_input.set_text(&path.to_string_lossy());
                } else {
                    nwg::modal_info_message(
                        &window,
                        i18n::alert_no_vaults(),
                        i18n::alert_no_vaults_body(),
                    );
                }
            }
            nwg::Event::OnButtonClick if handle == save_h => {
                let target = idx_to_target(target_combo.selection());
                let language = idx_to_language(lang_combo.selection());

                let draft = SettingsDraft {
                    vault_path: vault_input.text(),
                    target,
                    folder: folder_input.text(),
                    append_file: append_input.text(),
                    filename_template: tpl_input.text(),
                    prepend_timestamp_header: is_checked(&ts_check),
                    hotkey_raw: hotkey_input.text(),
                    autostart: is_checked(&autostart_check),
                    auto_update: is_checked(&auto_update_check),
                    language,
                };

                match settings::validate_and_build(draft) {
                    Ok(new_cfg) => {
                        i18n::apply(&new_cfg.language);
                        if let Err(e) = config::save(&new_cfg) {
                            nwg::modal_error_message(
                                &window,
                                i18n::alert_save_failed(),
                                &format!("{e:#}"),
                            );
                        } else {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    Err(e) => show_validation_error(&window, e),
                }
            }
            _ => {}
        });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&evt_handler);
    Ok(())
}
