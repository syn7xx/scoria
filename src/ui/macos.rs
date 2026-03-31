//! Native macOS settings (AppKit). Runs as `scoria settings-gui` so it does not share the tray
//! process's Tao event loop.

use anyhow::Context;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSApplicationActivationPolicy,
    NSApplicationDelegate, NSAutoresizingMaskOptions, NSBackingStoreType, NSButton,
    NSControlStateValueOff, NSControlStateValueOn, NSModalResponseOK, NSOpenPanel, NSPopUpButton,
    NSStackView, NSTextField, NSUserInterfaceLayoutOrientation, NSWindow, NSWindowDelegate,
    NSWindowStyleMask,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSEdgeInsets, NSNotification, NSObject, NSObjectProtocol, NSPoint,
    NSProcessInfo, NSRect, NSSize, NSString,
};
use std::cell::OnceCell;

use crate::engine::config::{self, SaveTarget};
use crate::engine::settings::{self, SettingsDraft, SettingsValidationError};
use crate::i18n;

// ---------------------------------------------------------------------------
// UI helpers
// ---------------------------------------------------------------------------

/// Sets the process title for Activity Monitor / `ps` / Force Quit (bare binaries often show argv oddly).
pub fn set_process_name() {
    NSProcessInfo::processInfo().setProcessName(&NSString::from_str("Scoria"));
}

fn vstack(mtm: MainThreadMarker) -> Retained<NSStackView> {
    let s = NSStackView::new(mtm);

    s.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
    s.setSpacing(8.0);
    s
}

fn hstack(mtm: MainThreadMarker) -> Retained<NSStackView> {
    let s = NSStackView::new(mtm);

    s.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal);
    s.setSpacing(8.0);
    s
}

fn add_label(root: &NSStackView, text: &str, mtm: MainThreadMarker) {
    root.addArrangedSubview(&NSTextField::labelWithString(
        &NSString::from_str(text),
        mtm,
    ));
}

fn add_field(
    root: &NSStackView,
    label: &str,
    value: &str,
    mtm: MainThreadMarker,
) -> Retained<NSTextField> {
    add_label(root, label, mtm);

    let field = NSTextField::textFieldWithString(&NSString::from_str(value), mtm);

    root.addArrangedSubview(&*field);
    field
}

fn add_checkbox(label: &str, checked: bool, mtm: MainThreadMarker) -> Retained<NSButton> {
    let btn = unsafe {
        NSButton::checkboxWithTitle_target_action(&NSString::from_str(label), None, None, mtm)
    };

    btn.setState(if checked {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    });

    btn
}

fn show_alert(mtm: MainThreadMarker, title: &str, message: &str) {
    let alert = NSAlert::new(mtm);

    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(message));
    alert.addButtonWithTitle(ns_string!("OK"));
    alert.runModal();
}

// ---------------------------------------------------------------------------
// Delegate
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SettingsIvars {
    window: OnceCell<Retained<NSWindow>>,
    vault: OnceCell<Retained<NSTextField>>,
    folder: OnceCell<Retained<NSTextField>>,
    append_file: OnceCell<Retained<NSTextField>>,
    filename_template: OnceCell<Retained<NSTextField>>,
    hotkey: OnceCell<Retained<NSTextField>>,
    target_popup: OnceCell<Retained<NSPopUpButton>>,
    lang_popup: OnceCell<Retained<NSPopUpButton>>,
    ts_check: OnceCell<Retained<NSButton>>,
    autostart_check: OnceCell<Retained<NSButton>>,
    auto_update_check: OnceCell<Retained<NSButton>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SettingsIvars]
    struct SettingsDelegate;

    unsafe impl NSObjectProtocol for SettingsDelegate {}

    unsafe impl NSApplicationDelegate for SettingsDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let mtm = self.mtm();
            let app = notification.object()
                .expect("notification object")
                .downcast::<NSApplication>()
                .expect("NSApplication");

            let mut cfg = match config::load_or_create() {
                Ok(c) => c,
                Err(e) => {
                    show_alert(mtm, i18n::alert_no_config(), &format!("{e:#}"));
                    app.terminate(None);
                    return;
                }
            };

            i18n::apply(&cfg.language);

            if cfg.vault_path.as_os_str().is_empty() {
                if let Some(p) = config::best_vault() {
                    cfg.vault_path = p;
                }
            }

            let root = vstack(mtm);

            root.setEdgeInsets(NSEdgeInsets {
                top: 12.0,
                left: 12.0,
                bottom: 12.0,
                right: 12.0,
            });

            // Vault row: label + editable field + Browse + Detect
            add_label(&root, i18n::settings_vault(), mtm);

            let vault_row = hstack(mtm);
            let vault_field = NSTextField::textFieldWithString(
                &NSString::from_str(&cfg.vault_path.to_string_lossy()),
                mtm,
            );

            vault_row.addArrangedSubview(&*vault_field);

            for (title, sel) in [
                (i18n::settings_browse(), objc2::sel!(browseVault:)),
                (i18n::settings_detect(), objc2::sel!(detectVault:)),
            ] {
                vault_row.addArrangedSubview(&*unsafe {
                    NSButton::buttonWithTitle_target_action(
                        &NSString::from_str(title),
                        Some(self.as_ref()),
                        Some(sel),
                        mtm,
                    )
                });
            }
            root.addArrangedSubview(&*vault_row);

            // Save mode
            add_label(&root, i18n::settings_save_mode(), mtm);

            let popup = NSPopUpButton::new(mtm);

            popup.setPullsDown(false);
            popup.removeAllItems();

            for label in [i18n::save_target_new_file(), i18n::save_target_append()] {
                popup.addItemWithTitle(&NSString::from_str(label));
            }

            popup.selectItemAtIndex(match cfg.target {
                SaveTarget::NewFileInFolder => 0,
                SaveTarget::AppendToFile => 1,
            });
            root.addArrangedSubview(&*popup);

            // Text fields
            let folder_field =
                add_field(&root, i18n::settings_folder(), &cfg.folder, mtm);
            let append_field =
                add_field(&root, i18n::settings_append(), &cfg.append_file, mtm);
            let template_field =
                add_field(&root, i18n::settings_template(), &cfg.filename_template, mtm);

            // Checkboxes
            let ts = add_checkbox(i18n::settings_timestamp(), cfg.prepend_timestamp_header, mtm);

            root.addArrangedSubview(&*ts);

            let autostart = add_checkbox(i18n::settings_autostart(), cfg.autostart, mtm);

            root.addArrangedSubview(&*autostart);

            // Auto-update checkbox
            let auto_update = add_checkbox(i18n::settings_auto_update(), cfg.auto_update, mtm);

            root.addArrangedSubview(&*auto_update);

            // Language
            add_label(&root, i18n::settings_lang(), mtm);

            let lang_popup = NSPopUpButton::new(mtm);

            lang_popup.setPullsDown(false);
            lang_popup.removeAllItems();

            for name in ["Auto / Авто", "English", "Русский"] {
                lang_popup.addItemWithTitle(&NSString::from_str(name));
            }

            lang_popup.selectItemAtIndex(match cfg.language.as_str() {
                "en" => 1,
                "ru" => 2,
                _ => 0,
            });
            root.addArrangedSubview(&*lang_popup);

            // Hotkey
            let hotkey_field = add_field(
                &root,
                i18n::settings_hotkey(),
                cfg.hotkey.as_deref().unwrap_or(""),
                mtm,
            );

            hotkey_field.setPlaceholderString(Some(&NSString::from_str(
                i18n::settings_hotkey_placeholder(),
            )));

            add_label(&root, i18n::settings_hotkey_hint(), mtm);

            // Action buttons
            let btn_row = hstack(mtm);

            for (title, sel) in [
                (i18n::settings_cancel(), objc2::sel!(cancelSettings:)),
                (i18n::settings_save(), objc2::sel!(saveSettings:)),
            ] {
                btn_row.addArrangedSubview(&*unsafe {
                    NSButton::buttonWithTitle_target_action(
                        &NSString::from_str(title),
                        Some(self.as_ref()),
                        Some(sel),
                        mtm,
                    )
                });
            }
            root.addArrangedSubview(&*btn_row);

            // Window
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(560.0, 580.0)),
                    NSWindowStyleMask::Titled
                        | NSWindowStyleMask::Closable
                        | NSWindowStyleMask::Miniaturizable
                        | NSWindowStyleMask::Resizable,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            unsafe { window.setReleasedWhenClosed(false) };

            window.setTitle(&NSString::from_str(i18n::settings_title()));

            let view = window.contentView().expect("content view");

            root.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );
            view.addSubview(&root);
            root.setFrame(view.bounds());
            window.setDelegate(Some(ProtocolObject::from_ref(self)));
            window.center();
            window.makeKeyAndOrderFront(None);

            // Store widget refs for later use in action methods
            let _ = self.ivars().window.set(window);
            let _ = self.ivars().vault.set(vault_field);
            let _ = self.ivars().folder.set(folder_field);
            let _ = self.ivars().append_file.set(append_field);
            let _ = self.ivars().filename_template.set(template_field);
            let _ = self.ivars().hotkey.set(hotkey_field);
            let _ = self.ivars().target_popup.set(popup);
            let _ = self.ivars().lang_popup.set(lang_popup);
            let _ = self.ivars().ts_check.set(ts);
            let _ = self.ivars().autostart_check.set(autostart);
            let _ = self.ivars().auto_update_check.set(auto_update);

            // Accessory: no Dock entry; window still works (same as tray Tao policy).
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }

    unsafe impl NSWindowDelegate for SettingsDelegate {
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }
    }

    impl SettingsDelegate {
        #[unsafe(method(browseVault:))]
        fn browse_vault(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let panel = NSOpenPanel::openPanel(mtm);

            panel.setCanChooseDirectories(true);
            panel.setCanChooseFiles(false);
            panel.setAllowsMultipleSelection(false);

            if panel.runModal() != NSModalResponseOK {
                return;
            }

            let urls = panel.URLs();

            if urls.count() == 0 {
                return;
            }

            if let (Some(path), Some(field)) =
                (urls.objectAtIndex(0).path(), self.ivars().vault.get())
            {
                field.setStringValue(&path);
            }
        }

        #[unsafe(method(detectVault:))]
        fn detect_vault(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let vaults = config::detect_obsidian_vaults();

            if vaults.is_empty() {
                show_alert(mtm, i18n::alert_no_vaults(), i18n::alert_no_vaults_body());
                return;
            }

            let idx = if vaults.len() == 1 {
                Some(0)
            } else {
                let alert = NSAlert::new(mtm);

                alert.setMessageText(&NSString::from_str(i18n::alert_choose_vault()));
                alert.setInformativeText(&NSString::from_str(i18n::alert_multiple_vaults()));

                for v in &vaults {
                    alert.addButtonWithTitle(&NSString::from_str(&format!(
                        "{}{}",
                        v.path.display(),
                        if v.open { i18n::alert_vault_open() } else { "" }
                    )));
                }

                alert.addButtonWithTitle(&NSString::from_str(i18n::settings_cancel()));

                let raw = alert.runModal() - NSAlertFirstButtonReturn;

                usize::try_from(raw).ok().filter(|&i| i < vaults.len())
            };

            if let (Some(i), Some(field)) = (idx, self.ivars().vault.get()) {
                field.setStringValue(&NSString::from_str(&vaults[i].path.to_string_lossy()));
            }
        }

        #[unsafe(method(saveSettings:))]
        fn save_settings(&self, _sender: Option<&AnyObject>) {
            let mtm = self.mtm();
            let iv = self.ivars();

            let target = match iv.target_popup.get().expect("popup").indexOfSelectedItem() {
                1 => SaveTarget::AppendToFile,
                _ => SaveTarget::NewFileInFolder,
            };

            let language = match iv.lang_popup.get().expect("lang").indexOfSelectedItem() {
                1 => "en".to_string(),
                2 => "ru".to_string(),
                _ => String::new(),
            };

            let draft = SettingsDraft {
                vault_path: iv.vault.get().expect("vault").stringValue().to_string(),
                target,
                folder: iv.folder.get().expect("folder").stringValue().to_string(),
                append_file: iv.append_file.get().expect("append").stringValue().to_string(),
                filename_template: iv
                    .filename_template
                    .get()
                    .expect("template")
                    .stringValue()
                    .to_string(),
                prepend_timestamp_header: iv.ts_check.get().expect("ts").state()
                    == NSControlStateValueOn,
                hotkey_raw: iv.hotkey.get().expect("hotkey").stringValue().to_string(),
                autostart: iv.autostart_check.get().expect("autostart").state()
                    == NSControlStateValueOn,
                auto_update: iv.auto_update_check.get().expect("auto_update").state()
                    == NSControlStateValueOn,
                language,
            };

            match settings::validate_and_build(draft) {
                Ok(new_cfg) => {
                    i18n::apply(&new_cfg.language);

                    match config::save(&new_cfg) {
                        Ok(()) => {
                            if let Some(w) = iv.window.get() {
                                w.performClose(None);
                            }
                        }
                        Err(e) => show_alert(mtm, i18n::alert_save_failed(), &format!("{e:#}")),
                    }
                }
                Err(e) => match e {
                    SettingsValidationError::EmptySubfolder => {
                        show_alert(mtm, i18n::alert_invalid(), i18n::alert_empty_subfolder());
                    }
                    SettingsValidationError::EmptyAppend => {
                        show_alert(mtm, i18n::alert_invalid(), i18n::alert_empty_append());
                    }
                    SettingsValidationError::EmptyTemplate => {
                        show_alert(mtm, i18n::alert_invalid(), i18n::alert_empty_template());
                    }
                    SettingsValidationError::InvalidHotkey(msg) => {
                        show_alert(mtm, i18n::alert_invalid_hotkey(), &msg);
                    }
                    SettingsValidationError::InvalidPath(msg) => {
                        show_alert(mtm, "Invalid path", &msg);
                    }
                },
            }
        }

        #[unsafe(method(cancelSettings:))]
        fn cancel_settings(&self, _sender: Option<&AnyObject>) {
            if let Some(w) = self.ivars().window.get() {
                w.performClose(None);
            }
        }
    }
);

impl SettingsDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(SettingsIvars::default());
        unsafe { msg_send![super(this), init] }
    }
}

pub fn run_blocking() -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new().context("settings GUI must run on the main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    let delegate = SettingsDelegate::new(mtm);

    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    app.run();

    Ok(())
}
