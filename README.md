# TermAVG (TMJ)

> [中文版说明](doc/README_zh.md)

<p align="center">
  <img src="doc/logo.png" alt="TMJ Logo" width="200" height="80">
</p>

<p align="center">
  A pixel-art visual novel (AVG) engine written in Rust, rendered in the terminal.<br />
  Driven by a script interpreter, supporting dialogue, characters, audio, and save/load.<br />
  <a href="https://github.com/rabitank/TerminalLove">See the example game "TerminalLove"</a>
</p>

## Features

<p align="center">
  <img src="doc/example.png" alt="Windows Terminal screenshot" width="480" height="320">
</p>

- **Dual rendering modes**: Terminal TUI (`tmj_terminal`) + GPU-accelerated window (`tmj_egui`)
- **Terminal rendering**: TUI drawing and event handling via `ratatui` + `crossterm`
- **GPU rendering**: Fullscreen, 1:2 character grid, CJK font support via `eframe` + `soft_ratatui`
- **Script-driven**: Built-in script parser with assignment, calls, `wait`, chaining, and more
- **Modular workspace**: `tmj_app`, `tmj_core`, `tmj_macro`, `tmj_terminal`, `tmj_egui`
- **Configurable startup**: Resolution, font, resource paths, and layout via `setting.toml`
- **Audio & resources**: Character sprites, expressions, and audio playback

## Quick Start

### Prerequisites

- Rust toolchain (stable, supporting `edition = "2024"`)
- Windows / Linux / macOS terminal

### Clone & Build

```bash
git clone https://github.com/rabitank/TermAVG.git
cd TermAVG
cargo build
```

### Run

Terminal TUI mode (redirect logs to avoid UI conflicts):

```bash
cargo run -p tmj_terminal 2> debug.txt
```

GPU window mode (fullscreen, CJK font support):

```bash
cargo run -p tmj_egui
```

> If `setting.toml` is missing on first run, it will be created with defaults automatically.

## Project Structure

```text
engine/
├─ tmj_app/            # Game logic, pages, script variables, rendering pipeline
├─ tmj_core/           # Script system, event system, resource paths, common utilities
├─ tmj_macro/          # Procedural macros
├─ tmj_terminal/       # TUI mode entry (crossterm terminal rendering)
├─ tmj_egui/           # GPU mode entry (eframe + soft_ratatui window rendering)
├─ resource/           # Scripts and assets (scripts, fonts, images, etc.)
├─ setting.toml        # Runtime configuration
├─ layout.toml         # Layout configuration
└─ README.md
```

## Configuration

### setting.toml

Example fields:

```toml
resolution = [240, 67]
font = "resource/font/SarasaTermCL-Regular.ttf"
font_bold = "resource/font/SarasaTermCL-Bold.ttf"
preprogress_script = ["resource/script_example.fs"]
is_force_skipable = false
save_dir = "save"
gallery_dir = "resource/gallery"
about_file = "resource/about.txt"
entre_script = "resource/script_example.fss"
mainmenu_title_file = "resource/mainmenu_title.txt"
mainmenu_default_bg_img = "resource/main_menu_bg.png"
mainmenu_session_bg_map = []
default_bg_img = "resource/bg_0.png"
default_face_img = "resource/default_face_img.png"
max_history_ls = 60
```

| Field | Description |
|-------|-------------|
| `resolution` | Logical render resolution `[w, h]` (character cells); the main画面 is centered accordingly |
| `font` | GPU mode font path (CJK monospace recommended, e.g. Sarasa Term CL) |
| `font_bold` | GPU mode bold font path (optional) |
| `preprogress_script` | Scripts to preprocess: `*.fs` → numbered `*.fss` on startup |
| `is_force_skipable` | Reserved, not yet used |
| `save_dir` | Save directory (normal slots + `temp.save`) |
| `gallery_dir` | Gallery resource directory |
| `about_file` | About popup content file (each line centered) |
| `entre_script` | Entry script path (usually points to preprocessed `*.fss`) |
| `mainmenu_title_file` | Main menu title file (optional, falls back to default) |
| `mainmenu_default_bg_img` | Main menu default background |
| `mainmenu_session_bg_map` | Session-based background mapping: `session_id_min`, `session_id_max`, `bg_img` |
| `default_bg_img` / `default_face_img` | Legacy fields, not directly read by the main flow |
| `max_history_ls` | Legacy field, history limit is internally hardcoded |

> All paths are relative to the project root.

### layout.toml

Defines the logical coordinate layout for story pages, main menu, and popups.

Coordinate shorthands:
- `ltwh`: `(left, top, width, height)`
- `twh`: `(top, width, height)` (horizontally centered, x derived from character count and spacing)
- `lw`: `(left, width)`
- `wh`: `(width, height)`

| Field | Meaning |
|-------|---------|
| `character_twh` | Character sprite box `(top, width, height)` |
| `two_character_spec` | Spacing for 2 characters on screen |
| `x_character_spec` | Spacing for 3+ characters on screen |
| `vertical_dark_edge` | Top/bottom dark bar height |
| `frame_face_ltwh` | Avatar area |
| `frame_content_ltwh` | Dialogue box body |
| `text_ltwh` | Text area (clipped inside `frame_content`) |
| `frame_name_ltwh` | Speaker name area |
| `short_key_ltwh` | Bottom shortcut bar area |
| `chapter_title_ltwh` | Chapter title area |
| `chapter_subtitle_ltwh` | Chapter subtitle area |
| `paragraph_ltwh` | Narration / paragraph text area |
| `history_wh` | History popup dimensions |
| `mainmenu_lw` | Main menu list panel `(left, width)` |
| `mainmenu_load_pop_lw` | Load popup `(left, width)`; `width = 0` uses remaining space |

## Script System

### File Format

Scripts use `#number` as section separators (e.g. `#1`, `#2`). The engine reads story content section by section. Entry scripts should use `*.fss`. If you maintain `*.fs` (no numbers after `#`), set `preprogress_script` to auto-number them into `*.fss` on startup.

### Execution Flow

1. **Preprocessing (optional)**: `Game::new()` iterates `preprogress_script`, converts `#` markers to `#1`/`#2`/..., outputs to `resource/<name>.fss`.
2. **Initialize script context**: Creates `ScriptContext`, registers global variables, functions, types (`character`, `text_obj`, etc.) and behavior mappings. Entry via `entre_script`.
3. **Streaming section reader**: `StreamSectionReader` reads one section by `session_id` (from `#N` to next marker). Returns EOF at end of file.
4. **Parse commands**: `ScriptParser` (lexer + parser) converts section text into command sequences (`set` / `once` / `wait` / `call` / `assignment` / `chain`).
5. **Frame-by-frame execution**: `Interpreter` injects commands into `SessionExecutor`, executed each frame via `update()`. `wait` pauses for time or input; `once` changes auto-revert at section end.

### Types

| Type | Description |
|------|-------------|
| `character` | Character object; use `character.say(text)` to drive dialogue, with sprite, expression, position, etc. |
| `layer` | Dynamic layer object; manage via the `layers` global, supports visual effects (alpha fade, glitch, heartbeat, etc.) |

### Global Objects

| Object | Description |
|--------|-------------|
| `bg` | Background state (image, edge toggle) |
| `bgm` | Background music state |
| `env_effect` | Ambient sound state |
| `frame` | Dialogue box state (visibility, content, typewriter params, etc.) |
| `paragraph` | Narration / long text area state |
| `chapter` | Chapter title / subtitle state |
| `character_ls` | Currently displayed character list |
| `layers` | Dynamic layer table (add/remove) |

### Global Functions

| Function | Description |
|----------|-------------|
| `text(content)` | Writes to `frame.content`, drives `FrameBehaviour` (narration mode, no speaker or avatar) |
| `voice(path, [seconds], [volume])` | Play audio; `seconds>0` fades out then in; `volume: 0~1`; empty string stops the track |
| `see(name)` | Print current info of a visual element (debug) |
| `log(path_or_expr)` | Print a script path value (debug) |
| `save_to(table, target_path)` | Serialize a script table to file |
| `create_default_character(path)` | Generate a default character config template |

### Object Methods

- `bg.set(path)` / `bg.trans_to(path, duration)` / `bg.show_edge()` / `bg.hide_edge()`
- `bgm.set(path, [fade_type], [seconds], [volume])` / `bgm.stop([seconds])`
- `env_effect.set(path, [seconds], [volume])` / `env_effect.stop([seconds])`
- `frame.show()` / `frame.hide()` / `frame.set_mode(mode)`
- `paragraph.show()` / `paragraph.hide()` / `paragraph.print(text)` / `paragraph.new(text)` / `paragraph.clear()`
- `chapter.show_title(title, [duration])` / `chapter.show_sub_title(subtitle, [duration])`
- `character.say(text)` (instance method)
- `character_ls.set_characters(c1, c2, ...)`

### Command Syntax

| Syntax | Description |
|--------|-------------|
| `变量 = 值` | Assignment |
| `变量 = 命令 参数...` | Command return value assignment |
| `对象.方法 参数...` | Method call |
| `set 路径 参数...` | Set value |
| `once 路径 参数...` | One-shot command, auto-reverted at section end |
| `wait 0.5` | Wait for specified seconds |
| `命令1 -> 命令2` | Chained call |

> Runtime environment info is written to `script_env.txt` on startup for debugging.
> See the [example script](https://github.com/rabitank/TermAVG/blob/main/resource/script_example.fs).

## Development

- The workspace contains multiple crates; run `cargo check` / `cargo test` from the project root
- Core script code is in `tmj_core/src/script/`
- Engine pages and rendering are in `tmj_app/src/pages/` and `tmj_app/src/game.rs`

### Hotkeys

| Key | Function |
|-----|----------|
| `↑/↓` or `←/→` | Move / select |
| `Enter` | Confirm / continue |
| `Esc` or `q` | Back / quit |

A shortcut bar is displayed at the bottom of every page.

## Dependencies

- **TUI**: [ratatui](https://github.com/ratatui/ratatui), [crossterm](https://github.com/crossterm-rs/crossterm)
- **GPU rendering**: [eframe](https://github.com/emilk/egui), [soft_ratatui](https://github.com/gold-silver-copper/soft_ratatui)
- **Serialization**: [serde](https://github.com/serde-rs/serde), [toml](https://github.com/toml-rs/toml)
- **Audio**: [rodio](https://github.com/RustAudio/rodio)
- **Other**: `tracing`, `anyhow`, `image`, `strum`, `fontdue`, `cosmic-text`

## Contributing

Issues and PRs are welcome.

## License

This project is open-sourced under the [MIT license](LICENSE).  
Copyright (c) 2024 rabitank
