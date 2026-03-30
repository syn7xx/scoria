use anyhow::Result;
use tray_icon::menu::{Menu, MenuItem};

use crate::i18n;

pub(crate) const MENU_SAVE: &str = "scoria.save";
pub(crate) const MENU_SETTINGS: &str = "scoria.settings";
pub(crate) const MENU_CONFIG: &str = "scoria.config";
pub(crate) const MENU_UPDATE: &str = "scoria.update";
pub(crate) const MENU_QUIT: &str = "scoria.quit";

pub(crate) struct MenuItems {
    pub(crate) save: MenuItem,
    pub(crate) settings: MenuItem,
    pub(crate) config_item: MenuItem,
    pub(crate) update: MenuItem,
    pub(crate) quit: MenuItem,
}

impl MenuItems {
    pub(crate) fn build() -> Result<(Menu, Self)> {
        let save = MenuItem::with_id(MENU_SAVE, i18n::menu_save(), true, None);
        let settings = MenuItem::with_id(MENU_SETTINGS, i18n::menu_settings(), true, None);
        let config_item = MenuItem::with_id(MENU_CONFIG, i18n::menu_config(), true, None);
        let update = MenuItem::with_id(MENU_UPDATE, i18n::menu_update(), true, None);
        let quit = MenuItem::with_id(MENU_QUIT, i18n::menu_quit(), true, None);
        let menu = Menu::new();

        menu.append(&save)?;
        menu.append(&settings)?;
        menu.append(&config_item)?;
        menu.append(&update)?;
        menu.append(&quit)?;

        Ok((
            menu,
            Self {
                save,
                settings,
                config_item,
                update,
                quit,
            },
        ))
    }

    pub(crate) fn refresh_labels(&self) {
        self.save.set_text(i18n::menu_save());
        self.settings.set_text(i18n::menu_settings());
        self.config_item.set_text(i18n::menu_config());
        self.update.set_text(i18n::menu_update());
        self.quit.set_text(i18n::menu_quit());
    }
}
