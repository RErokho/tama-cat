# Desktop Cat

Живой кот на Windows (Shimeji-like). **Rust**, без Electron.

## Сборка и запуск

Батники (из корня репо):

| Файл | Что делает |
| --- | --- |
| `run.bat` | `cargo run -p desktop-cat` |
| `build.bat` | debug-сборка → `target\debug\desktop-cat.exe` |
| `build-release.bat` | release-сборка → `target\release\desktop-cat.exe` |
| `run-release.bat` | собрать release при необходимости и запустить exe |

```bash
cargo run -p desktop-cat
cargo build -p desktop-cat --release
```

Бинарник: `target/debug/desktop-cat.exe` или `target/release/desktop-cat.exe`.

Спрайты: `assets/sprites/cat/sheet.png` + `sheet.json`  
Переопределение: `DESKTOP_CAT_ASSETS=...`

Проверка спрайтов (цвет / размер / fill% / правила):

```bash
scripts\check_sprites.bat
scripts\check_sprites_new.bat
scripts\check_fill.bat
python scripts/check_sprites.py --stems lookDown-0 jumpDown-0
python scripts/build_scale_profile.py
```

Карта масштаба: `assets/sprites/cat/scale_profile.json` (fill%, mass_h, pose bands).  
Новый кадр с зумом по карте: `python scripts/process_sprite_frame.py raw.jpg -o clean/STEM.png --use-profile`

## Crates

| Crate | Роль |
| --- | --- |
| `cat-core` | физика, movement, behavior |
| `cat-render` | sprite sheet → BGRA |
| `cat-platform` | layered HWND, мониторы, EnumWindows |
| `desktop-cat` | binary |

## Управление

- ЛКМ — тащить / кинуть  
- ПКМ — вернуть на основной монитор  

Поведение: `docs/BEHAVIOR.md`
