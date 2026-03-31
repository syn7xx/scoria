use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, ButtonsType, CheckButton, ComboBoxText, DialogFlags, Entry,
    FileChooserAction, FileChooserDialog, Label, MessageDialog, MessageType, Orientation,
    PolicyType, ResponseType, ScrolledWindow, Separator, Window, WindowPosition, WindowType,
};

use crate::engine::config::{self, SaveTarget};
use crate::engine::settings::{self, SettingsDraft, SettingsValidationError};
use crate::i18n;

fn alert(parent: &Window, kind: MessageType, msg: &str) {
    let d = MessageDialog::new(
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        kind,
        ButtonsType::Ok,
        msg,
    );

    d.run();
    d.close();
}

fn heading(text: &str) -> Label {
    let l = Label::new(Some(text));

    l.set_halign(gtk::Align::Start);
    l
}

pub fn open() {
    if !gtk::is_initialized() && gtk::init().is_err() {
        eprintln!("scoria: GTK init failed");
        return;
    }

    let cfg = match config::load_or_create() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("scoria: could not load config: {e:#}");
            return;
        }
    };
    i18n::apply(&cfg.language);

    let window = Window::new(WindowType::Toplevel);

    window.set_title(i18n::settings_title());
    window.set_default_size(560, -1);
    window.set_position(WindowPosition::Center);

    let scroll = ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);

    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    let root = GtkBox::new(Orientation::Vertical, 10);

    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    // --- Vault path ---
    root.pack_start(&heading(i18n::settings_vault()), false, false, 0);

    let vault_row = GtkBox::new(Orientation::Horizontal, 8);
    let vault_entry = Entry::new();

    vault_entry.set_hexpand(true);
    vault_entry.set_text(&cfg.vault_path.to_string_lossy());

    if cfg.vault_path.as_os_str().is_empty() {
        if let Some(p) = config::best_vault() {
            vault_entry.set_text(&p.to_string_lossy());
        }
    }

    let detect_btn = Button::with_label(i18n::settings_detect());
    let browse_btn = Button::with_label(i18n::settings_browse());

    vault_row.pack_start(&vault_entry, true, true, 0);
    vault_row.pack_start(&detect_btn, false, false, 0);
    vault_row.pack_start(&browse_btn, false, false, 0);
    root.pack_start(&vault_row, false, false, 0);

    // --- Save mode ---
    root.pack_start(&heading(i18n::settings_save_mode()), false, false, 0);

    let target_combo = ComboBoxText::new();

    for (id, label) in [
        (
            SaveTarget::NewFileInFolder.as_id(),
            i18n::save_target_new_file(),
        ),
        (SaveTarget::AppendToFile.as_id(), i18n::save_target_append()),
    ] {
        target_combo.append(Some(id), label);
    }

    target_combo.set_active_id(Some(cfg.target.as_id()));
    root.pack_start(&target_combo, false, false, 0);

    // --- Text fields ---
    let folder_entry = labeled_entry(&root, i18n::settings_folder(), &cfg.folder);
    let append_entry = labeled_entry(&root, i18n::settings_append(), &cfg.append_file);
    let template_entry = labeled_entry(&root, i18n::settings_template(), &cfg.filename_template);

    // --- Timestamp checkbox ---
    let ts_check = CheckButton::with_label(i18n::settings_timestamp());

    ts_check.set_active(cfg.prepend_timestamp_header);
    root.pack_start(&ts_check, false, false, 0);

    // --- Autostart checkbox ---
    let autostart_check = CheckButton::with_label(i18n::settings_autostart());

    autostart_check.set_active(cfg.autostart);
    root.pack_start(&autostart_check, false, false, 0);

    // --- Auto-update checkbox ---
    let auto_update_check = CheckButton::with_label(i18n::settings_auto_update());

    auto_update_check.set_active(cfg.auto_update);
    root.pack_start(&auto_update_check, false, false, 0);

    // --- Language ---
    root.pack_start(&heading(i18n::settings_lang()), false, false, 0);

    let lang_combo = ComboBoxText::new();

    lang_combo.append(Some(""), "Auto / Авто");
    lang_combo.append(Some("en"), "English");
    lang_combo.append(Some("ru"), "Русский");

    let lang_id: &str = if cfg.language.is_empty() {
        ""
    } else {
        &cfg.language
    };

    lang_combo.set_active_id(Some(lang_id));
    root.pack_start(&lang_combo, false, false, 0);

    // --- Hotkey ---
    root.pack_start(&heading(i18n::settings_hotkey_x11()), false, false, 0);

    let hotkey_entry = Entry::new();

    if let Some(ref h) = cfg.hotkey {
        hotkey_entry.set_text(h);
    }

    hotkey_entry.set_placeholder_text(Some(i18n::settings_hotkey_placeholder()));
    root.pack_start(&hotkey_entry, false, false, 0);

    let hint = Label::new(Some(i18n::settings_hotkey_wayland()));

    hint.set_line_wrap(true);
    hint.set_halign(gtk::Align::Start);
    root.pack_start(&hint, false, false, 0);

    // --- Buttons ---
    root.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);

    let btn_row = GtkBox::new(Orientation::Horizontal, 8);

    btn_row.set_halign(gtk::Align::End);

    let open_raw_btn = Button::with_label(i18n::settings_raw());
    let cancel_btn = Button::with_label(i18n::settings_cancel());
    let save_btn = Button::with_label(i18n::settings_save());

    btn_row.pack_start(&open_raw_btn, false, false, 0);
    btn_row.pack_end(&save_btn, false, false, 0);
    btn_row.pack_end(&cancel_btn, false, false, 0);
    root.pack_start(&btn_row, false, false, 0);

    scroll.add(&root);
    window.add(&scroll);

    // --- Handlers ---
    detect_btn.connect_clicked(clone!(@weak window, @weak vault_entry => move |_| {
        let vaults = config::detect_obsidian_vaults();

        if vaults.is_empty() {
            alert(&window, MessageType::Info, i18n::alert_no_vaults_gtk());
            return;
        }

        if vaults.len() == 1 {
            vault_entry.set_text(&vaults[0].path.to_string_lossy());
            return;
        }

        let dlg = MessageDialog::new(
            Some(&window),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Question,
            ButtonsType::None,
            i18n::alert_pick_vault(),
        );

        for (i, v) in vaults.iter().enumerate() {
            let label = format!(
                "{}{}",
                v.path.display(),
                if v.open { i18n::alert_vault_open() } else { "" }
            );
            dlg.add_button(&label, ResponseType::Other(i as u16));
        }

        dlg.add_button(i18n::settings_cancel(), ResponseType::Cancel);

        let resp = dlg.run();

        dlg.close();

        if let ResponseType::Other(idx) = resp {
            if let Some(v) = vaults.get(idx as usize) {
                vault_entry.set_text(&v.path.to_string_lossy());
            }
        }
    }));

    browse_btn.connect_clicked(clone!(@weak window, @weak vault_entry => move |_| {
        let dlg = FileChooserDialog::with_buttons(
            Some(i18n::alert_choose_vault()),
            Some(&window),
            FileChooserAction::SelectFolder,
            &[("_Cancel", ResponseType::Cancel), ("_Select", ResponseType::Accept)],
        );
        if dlg.run() == ResponseType::Accept {
            if let Some(f) = dlg.file() {
                if let Some(p) = f.path() {
                    vault_entry.set_text(&p.to_string_lossy());
                }
            }
        }
        dlg.close();
    }));

    open_raw_btn.connect_clicked(|_| config::open_in_editor());
    cancel_btn.connect_clicked(clone!(@weak window => move |_| window.close()));

    save_btn.connect_clicked(clone!(
        @weak window, @weak vault_entry, @weak target_combo,
        @weak folder_entry, @weak append_entry, @weak template_entry,
        @weak ts_check, @weak autostart_check, @weak auto_update_check,
        @weak hotkey_entry, @weak lang_combo
        => move |_| {
            let draft = SettingsDraft {
                vault_path: trimmed(&vault_entry),
                target: target_combo
                    .active_id()
                    .as_deref()
                    .and_then(SaveTarget::from_id)
                    .unwrap_or_default(),
                folder: trimmed(&folder_entry),
                append_file: trimmed(&append_entry),
                filename_template: trimmed(&template_entry),
                prepend_timestamp_header: ts_check.is_active(),
                hotkey_raw: trimmed(&hotkey_entry),
                autostart: autostart_check.is_active(),
                auto_update: auto_update_check.is_active(),
                language: lang_combo
                    .active_id()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            };

            match settings::validate_and_build(draft) {
                Ok(new_cfg) => {
                    // Apply immediately so subsequent dialogs / notifications in this
                    // session use the new language without waiting for the file watcher.
                    i18n::apply(&new_cfg.language);

                    match config::save(&new_cfg) {
                        Ok(()) => window.close(),
                        Err(e) => alert(
                            &window,
                            MessageType::Error,
                            &format!("{}:\n{e:#}", i18n::alert_save_failed()),
                        ),
                    }
                }
                Err(e) => match e {
                    SettingsValidationError::EmptySubfolder => {
                        alert(&window, MessageType::Warning, i18n::alert_empty_subfolder());
                    }
                    SettingsValidationError::EmptyAppend => {
                        alert(&window, MessageType::Warning, i18n::alert_empty_append());
                    }
                    SettingsValidationError::EmptyTemplate => {
                        alert(&window, MessageType::Warning, i18n::alert_empty_template());
                    }
                    SettingsValidationError::InvalidHotkey(msg) => {
                        alert(
                            &window,
                            MessageType::Error,
                            &format!("{}: {msg}", i18n::alert_invalid_hotkey()),
                        );
                    }
                    SettingsValidationError::InvalidPath(msg) => {
                        alert(
                            &window,
                            MessageType::Error,
                            &format!("Invalid path: {msg}"),
                        );
                    }
                },
            }
        }
    ));

    window.show_all();
    window.present();
}

fn labeled_entry(container: &GtkBox, label: &str, initial: &str) -> Entry {
    container.pack_start(&heading(label), false, false, 0);
    let entry = Entry::new();
    entry.set_text(initial);
    container.pack_start(&entry, false, false, 0);
    entry
}

fn trimmed(entry: &Entry) -> String {
    entry.text().as_str().trim().to_string()
}
