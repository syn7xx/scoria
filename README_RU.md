# Scoria

[English](README.md)

Сохраняет содержимое буфера обмена — текст **и изображения** — в хранилище [Obsidian](https://obsidian.md/).
Работает как приложение в **системном трее** на Linux, macOS и Windows и как **CLI**-утилита.

Scoria умная: на Linux сначала проверяет **выделенный текст** (primary selection), затем **буфер обмена**. На macOS читает системный буфер. Linux: выделите или скопируйте, затем сохраните. macOS: сначала скопируйте (`Cmd+C`), затем сохраните.

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

Linux/macOS: установка или обновление одной командой:

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/install.sh | bash
```

Или вручную со страницы [Releases](https://github.com/syn7xx/scoria/releases).

#### Windows

Для простой установки используйте `scoria-windows-x86_64.msi` со страницы [Releases](https://github.com/syn7xx/scoria/releases).
Мастер спрашивает папку установки и, нужен ли ярлык на **рабочем столе**; ярлык в **меню «Пуск»** создаётся всегда. Отдельного экрана с лицензией нет.

Портативный вариант: скачайте `scoria-windows-x86_64.zip`, распакуйте `scoria.exe` и добавьте папку с ним в `PATH`.
`install.sh` / `uninstall.sh` работают только на Unix-подобных системах.

Или установка portable ZIP одной командой PowerShell:

```powershell
irm https://github.com/syn7xx/scoria/raw/main/install.ps1 | iex
```

Для мейнтейнеров: можно сгенерировать winget-манифесты (установщик — **портативный ZIP**) по SHA256 **архива** `scoria-windows-x86_64.zip`:

```powershell
powershell -File scripts/windows/gen-winget-manifests.ps1 -Version 0.2.4 -Sha256 "<sha256 scoria-windows-x86_64.zip>"
```

Версию укажите как в релизе (см. `version` в `Cargo.toml`). Скрипт снимает ведущий `v` у `-Version`, если он есть.

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

Linux/macOS одной командой (как при установке через `install.sh`):

```bash
curl -sL https://github.com/syn7xx/scoria/raw/main/uninstall.sh | bash
```

На Windows (portable ZIP-установка):

```powershell
irm https://github.com/syn7xx/scoria/raw/main/uninstall.ps1 | iex
```

На Windows (MSI-установка): удалите через **Параметры → Приложения** (предпочтительно) или `msiexec /x` с **полным путём** к установленному MSI, например:

```powershell
msiexec /x "$env:USERPROFILE\Downloads\scoria-windows-x86_64.msi"
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

На Windows команда `scoria settings-gui` открывает нативное окно настроек. Запуск трея из меню «Пуск» или автозагрузки **не** открывает окно консоли.

Чтобы править сырой TOML, используйте пункт **Open config file…** в меню трея.

При первом запуске Scoria:
- создаст файл конфигурации (`~/.config/scoria/config.toml` на Linux, `~/Library/Application Support/scoria/config.toml` на macOS, `%APPDATA%\\scoria\\config.toml` на Windows)
- автоматически определит хранилище Obsidian
- выберет **язык интерфейса** по локали системы (сменить можно в **Настройках…**)
- покажет иконку в трее (при запуске `scoria` / `scoria run`)

## Использование

### Меню трея

Подписи пунктов зависят от [языка интерфейса](#язык-интерфейса) (по умолчанию — как в системе). Типичные варианты:

| Пункт (EN / RU) | Действие |
|-----------------|----------|
| **Save to Obsidian** / **Сохранить в Obsidian** | Сохранить выделение или буфер |
| **Settings…** / **Настройки…** | GTK (Linux) / AppKit (macOS) / Win32 (Windows) |
| **Open config file…** / **Открыть файл конфигурации…** | Редактор с `config.toml` |
| **Check for updates** / **Проверить обновления** | Linux/macOS: скачать и установить последнюю версию; Windows: показать уведомление и открыть страницу последних релизов для ручного обновления MSI/winget |
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

**Windows**

Используйте PowerToys Keyboard Manager, AutoHotkey или другой менеджер горячих клавиш:
- Команда: `scoria save`

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
| `hotkey` | *(нет)* | Глобальная комбинация (X11 и Windows в режиме трея) |
| `autostart` | `false` | Запускать Scoria при входе в систему |
| `auto_update` | `false` | Автоматически проверять обновления при запуске трея |
| `language` | *(пусто)* | Язык интерфейса: пустая строка = авто по `LANG` / `LC_MESSAGES` / `LC_ALL`, либо `en`, `ru` |

### Язык интерфейса

Доступны **английский** и **русский** (меню трея, уведомления, окно настроек).

- В **Настройках…** поле **Язык интерфейса**: *Auto / Авто*, *English* или *Русский*. После сохранения язык меняется сразу в этом окне; трей обновляет подписи меню и подсказку вскоре после записи `config.toml`.
- В файле конфига: `language = ""` (авто), `language = "en"` или `language = "ru"`.

### Автозапуск

Включите **«Start Scoria on login»** в настройках (или `autostart = true` в файле конфига). Scoria:

- **Linux**: создаст `.desktop`-запись в `~/.config/autostart/`
- **macOS**: создаст LaunchAgent в `~/Library/LaunchAgents/`
- **Windows**: добавит/удалит `Scoria` в `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

Снятие галочки удаляет запись автозапуска.

## Обновление

Scoria может автоматически проверять обновления при запуске трея, если `auto_update = true` (по умолчанию выключено). Ручная проверка всегда доступна через меню трея.

Linux/macOS: нажмите **«Check for updates»** в меню трея — Scoria скачает и заменит бинарник автоматически. Перезапустите для применения.

Windows: обновляйте через MSI/winget (встроенная замена бинарника отключена для корректной MSI-установки). Команда в трее **«Check for updates»** показывает уведомление и открывает страницу [Releases](https://github.com/syn7xx/scoria/releases/latest).

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

## Внесение вклада

Мы приветствуем вклады! Подробности в [CONTRIBUTING_RU.md](CONTRIBUTING_RU.md).

## Лицензия

MIT OR Apache-2.0
