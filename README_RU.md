# Scoria

[English](README.md)

Сохраняет содержимое буфера обмена — текст **и изображения** — в хранилище [Obsidian](https://obsidian.md/).
Работает как приложение в **системном трее** на Linux и macOS и как **CLI**-утилита.

Scoria умная: на Linux сначала проверяет **выделенный текст** (primary selection), затем **буфер обмена**. На macOS читает системный буфер. Просто выделите или скопируйте — и сохраните одним действием.

## Установка

### Из исходников (все платформы)

```bash
git clone https://github.com/syn7xx/scoria.git
cd scoria
make deps      # установит системные библиотеки (автоопределение pacman / dnf / apt / brew)
make install   # соберёт и установит в ~/.local/bin (+ иконки и .desktop на Linux)
```

Убедитесь, что `~/.local/bin` в вашем `PATH`.

### Готовый бинарник (рекомендуется)

Одна команда для установки или обновления:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

Или вручную со страницы [Releases](https://github.com/syn7xx/scoria/releases).

### Через cargo

```bash
cargo install --git https://github.com/syn7xx/scoria.git
```

Устанавливает только бинарник. На Linux сначала нужно установить системные библиотеки (см. ниже).

### Системные зависимости

#### Linux

| Дистрибутив | Команда |
|-------------|---------|
| **Arch / Omarchy** | `sudo pacman -S --needed gtk3 libappindicator-gtk3 xdotool wl-clipboard` |
| **Fedora** | `sudo dnf install gtk3-devel libappindicator-gtk3-devel xdotool wl-clipboard` |
| **Ubuntu / Debian** | `sudo apt install libgtk-3-dev libappindicator3-dev libxdo-dev xdotool wl-clipboard` |

> **GNOME**: для иконки в трее нужно расширение [AppIndicator](https://extensions.gnome.org/extension/615/appindicator-support/) (пакет `gnome-shell-extension-appindicator`). Без него `scoria save` по-прежнему работает через горячую клавишу.

#### macOS

Дополнительных зависимостей не нужно — всё через нативные API.

## Удаление

Удаляет бинарник в `~/.local/bin`, иконки и `.desktop` на Linux, конфиг (`~/.config/scoria` на Linux, `~/Library/Application Support/scoria` на macOS) и автозапуск. По возможности завершает запущенный `scoria`.

Из клона репозитория:

```bash
./uninstall.sh
```

Одной командой (как при установке через `install.sh`):

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/uninstall.sh | bash
```

Если ставили через `make install`, можно выполнить `make uninstall` (только бинарник, иконки и ярлык; полное удаление с конфигом и автозапуском — скриптом выше).

## Быстрый старт

```bash
scoria        # запускает трей (по умолчанию)
scoria save   # разовое сохранение выделения / буфера в Obsidian
```

На macOS то же окно настроек можно открыть из терминала (короткоживущий вспомогательный процесс):

```bash
scoria settings-gui
```

На Linux (в том числе GNOME без AppIndicator) настройки также можно открыть из терминала:

```bash
scoria settings-gui
```

Чтобы править сырой TOML, используйте пункт **Open config file…** в меню трея или откройте файл  
`~/Library/Application Support/scoria/config.toml` в любом редакторе (например `open -e` для TextEdit).

При первом запуске Scoria:
- создаст файл конфигурации (`~/.config/scoria/config.toml` на Linux, `~/Library/Application Support/scoria/config.toml` на macOS)
- автоматически определит хранилище Obsidian
- выберет **язык интерфейса** по локали системы (сменить можно в **Настройках…**)
- покажет иконку в трее (при запуске `scoria` / `scoria run`)

## Использование

### Меню трея

Подписи пунктов зависят от [языка интерфейса](#язык-интерфейса) (по умолчанию — как в системе). Типичные варианты:

| Пункт (EN / RU) | Действие |
|-----------------|----------|
| **Save to Obsidian** / **Сохранить в Obsidian** | Сохранить выделение или буфер |
| **Settings…** / **Настройки…** | GTK (Linux) / AppKit (macOS) |
| **Open config file…** / **Открыть файл конфигурации…** | Редактор с `config.toml` |
| **Check for updates** / **Проверить обновления** | Скачать и установить новую версию |
| **Quit** / **Выход** | Завершить приложение |

### Что сохраняется

| Контент | Результат в хранилище |
|---------|----------------------|
| **Текст** | Markdown-файл в `folder/` (или дописывается в `append_file`) |
| **Изображение** (PNG, JPEG, WebP, GIF, BMP, SVG) | Файл в `folder/attachments/`, плюс `.md`-заметка с `![[...]]` |

### Горячие клавиши

Привяжите `scoria save` к хоткею в вашем окружении:

**Hyprland / Sway (Wayland)**

```ini
bind = $mainMod, V, exec, scoria save
```

**GNOME**

Настройки → Клавиатура → Пользовательские комбинации:
- Имя: `Scoria save`
- Команда: `scoria save`
- Комбинация: на ваш выбор (например `Super+V`)

**macOS**

Используйте [Hammerspoon](https://www.hammerspoon.org/), [skhd](https://github.com/koekeishiya/skhd) или Системные настройки → Клавиатура → Сочетания клавиш:

```lua
-- Пример для Hammerspoon
hs.hotkey.bind({"cmd", "shift"}, "V", function()
  os.execute("scoria save")
end)
```

**X11 (встроенный)**

Укажите `hotkey` в конфиге (например `hotkey = "Ctrl+Shift+S"`) — Scoria зарегистрирует её при работе трея. Модификаторы: `ctrl`, `alt`, `shift`, `super`; клавиши: `a`-`z`, `0`-`9`, `F1`-`F12`, `Space` и т.д.

## Конфигурация

Редактируйте через **Настройки…** в меню трея или напрямую в файле конфига.

| Поле | По умолчанию | Описание |
|------|-------------|----------|
| `vault_path` | *(автоопределение)* | Абсолютный путь к корню хранилища |
| `target` | `new_file_in_folder` | `new_file_in_folder` или `append_to_file` |
| `folder` | `scoria` | Подпапка для новых файлов |
| `append_file` | `Scoria.md` | Путь внутри хранилища при дописывании |
| `filename_template` | `clip-%Y-%m-%d-%H%M%S.md` | Шаблон имени (strftime) |
| `prepend_timestamp_header` | `true` | Добавлять заголовок `## timestamp` |
| `hotkey` | *(нет)* | Глобальная комбинация (только X11) |
| `autostart` | `false` | Запускать Scoria при входе в систему |
| `language` | *(пусто)* | Язык интерфейса: пустая строка = авто по `LANG` / `LC_MESSAGES` / `LC_ALL`, либо `en`, `ru` |

### Язык интерфейса

Доступны **английский** и **русский** (меню трея, уведомления, окно настроек).

- В **Настройках…** поле **Язык интерфейса**: *Auto / Авто*, *English* или *Русский*. После сохранения язык меняется сразу в этом окне; трей обновляет подписи меню и подсказку вскоре после записи `config.toml`.
- В файле конфига: `language = ""` (авто), `language = "en"` или `language = "ru"`.

### Автозапуск

Включите **«Start Scoria on login»** в настройках (или `autostart = true` в файле конфига). Scoria:

- **Linux**: создаст `.desktop`-запись в `~/.config/autostart/`
- **macOS**: создаст LaunchAgent в `~/Library/LaunchAgents/`

Снятие галочки удаляет запись автозапуска.

## Обновление

Scoria автоматически проверяет обновления при запуске трея. Когда доступна новая версия — появится уведомление.

Для обновления: нажмите **«Check for updates»** в меню трея — Scoria скачает и заменит бинарник автоматически. Перезапустите для применения.

Или повторите команду установки:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

## Разработка

```bash
make build    # cargo build --release
make check    # cargo clippy
make fmt      # cargo fmt
make clean    # cargo clean
```

## Лицензия

MIT OR Apache-2.0
