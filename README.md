# TermAVG (TMJ)

<!-- PROJECT SHIELDS -->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]

<p align="center">
  <img src="doc/logo.png" alt="TMJ Logo" width="200" height="80">
</p>

<p align="center">
  一个使用 Rust 编写、在终端渲染的像素风文字冒险（AVG）引擎。<br />
  引擎以脚本解释器驱动剧情，支持对话、角色、音频和存档流程。<br />
  <a href="https://github.com/rabitank/TerminalLove">查看游戏《终末之爱》示例项目</a>
</p>

## 目录

- [TermAVG (TMJ)](#termavg-tmj)
  - [目录](#目录)
  - [项目特性](#项目特性)
  - [项目结构](#项目结构)
  - [快速开始](#快速开始)
    - [环境要求](#环境要求)
    - [克隆与构建](#克隆与构建)
    - [运行](#运行)
  - [配置文件](#配置文件)
    - [setting 字段说明](#setting-字段说明)
  - [脚本说明](#脚本说明)
    - [脚本分段与文件格式](#脚本分段与文件格式)
    - [脚本读取与执行机制](#脚本读取与执行机制)
    - [脚本环境：对象、函数与方法](#脚本环境对象函数与方法)
    - [常见命令语法](#常见命令语法)
    - [最小示例](#最小示例)
  - [layout 布局说明](#layout-布局说明)
    - [坐标与尺寸约定](#坐标与尺寸约定)
    - [layout.toml 字段含义](#layouttoml-字段含义)
  - [开发说明](#开发说明)
  - [依赖](#依赖)
  - [贡献](#贡献)

## 项目特性
在 windows terminal上的效果:
<p align="center">
  <img src="doc/example.png" alt="windows terminal" width="480" height="320">
</p>


- **双运行模式**：终端 TUI 模式 (`tmj_terminal`) + GPU 加速窗口模式 (`tmj_egui`)
- 终端渲染：基于 `ratatui` + `crossterm` 的 TUI 绘制与事件处理。
- GPU 渲染：基于 `eframe` + `soft_ratatui`，支持全屏、1:2 字符网格、CJK 字体。
- 脚本驱动：内置脚本解析器，支持赋值、调用、`wait`、链式调用等语法。
- 多模块工作区：`tmj_app`、`tmj_core`、`tmj_macro`、`tmj_terminal`、`tmj_egui`。
- 可配置启动：通过 `setting.toml` 指定分辨率、字体路径、资源路径和布局参数。
- 音频与资源：支持角色立绘、表情资源和音频播放。

## 项目结构

```text
engine/
├─ tmj_app/            # 游戏逻辑、页面、脚本变量与渲染流程
├─ tmj_core/           # 脚本系统、事件系统、资源路径与通用能力
├─ tmj_macro/          # 过程宏
├─ tmj_terminal/       # TUI 模式入口（crossterm 终端渲染）
├─ tmj_egui/           # GPU 模式入口（eframe + soft_ratatui 窗口渲染）
├─ resource/           # 脚本与资源文件（脚本、字体、图片等）
├─ setting.toml        # 运行配置
├─ layout.toml         # 布局配置
└─ README.md
```

### 运行模式

**终端 TUI 模式**（原有）：
```bash
cargo run -p tmj_terminal 2> debug.txt
```

**GPU 窗口模式**（新增）：
```bash
cargo run -p tmj_egui
```
启动后自动全屏，使用 1:2 字符网格渲染，支持 CJK 字体。

## 快速开始

### 环境要求

- Rust 工具链（建议稳定版，支持 `edition = "2024"`）
- Windows / Linux / macOS 终端环境

### 克隆与构建

```bash
git clone https://github.com/rabitank/TermAVG.git
cd TermAVG
cargo build
```

### 运行

建议使用
```bash
cargo run 2> debug.txt
```
否则会出现debug模式下日志打印和游戏界面冲突

首次运行时如果没有 `setting.toml`，程序会按默认配置自动创建。

## 配置文件

`setting.toml` 关键字段示例：

```toml
resolution = [240, 67]
font = "resource/font/SarasaTermCL-Regular.ttf"
font_bold = "resource/font/SarasaTermCL-Bold.ttf"
preprogress_script = ["resource/script_example.fs"]
is_force_skipable = false
save_dir = "save"
gallery_dir = "resource/gallery"
about_file = "resource/about.txt" # 可选
entre_script = "resource/script_example.fss"
mainmenu_title_file = "resource/mainmenu_title.txt" # 可选
mainmenu_default_bg_img = "resource/main_menu_bg.png"
mainmenu_session_bg_map = []
default_bg_img = "resource/bg_0.png"
default_face_img = "resource/default_face_img.png"
max_history_ls = 60
```

### setting 字段说明

- `resolution: [w, h]`：逻辑渲染分辨率（字符格单位），游戏主画面会按这个尺寸居中绘制。
- `font`：GPU 模式使用的字体文件路径（推荐 CJK 等宽字体如 Sarasa Term CL）。
- `font_bold`：GPU 模式使用的粗体字体文件路径（可选）。
- `preprogress_script: []`：需要预处理的脚本列表。启动时会把这些 `*.fs` 转成带段号的 `*.fss`（见下文机制）。
- `is_force_skipable`：预留字段，当前版本尚未在运行时逻辑中消费。
- `save_dir`：存档目录（普通槽位与 `temp.save` 都在这里）。
- `gallery_dir`：图鉴目录路径，用于存放图鉴相关资源。
- `about_file`：主菜单 About 弹窗内容文件路径（每行作为一段文本居中显示）。
- `entre_script`：入口脚本路径（通常指向预处理后的 `*.fss`）。
- `mainmenu_title_file`：主菜单标题文本文件路径（可选，不填则用默认标题效果）。
- `mainmenu_default_bg_img`：主菜单默认背景图。
- `mainmenu_session_bg_map`：主菜单背景映射列表，按 `session_id` 区间匹配背景图：
  - `session_id_min` / `session_id_max`：生效区间。
  - `bg_img`：该区间使用的背景图。
- `default_bg_img` / `default_face_img`：历史兼容字段，当前主流程未直接读取，可按项目需要在自定义脚本/行为层使用。
- `max_history_ls`：历史兼容字段，当前历史上限由内部实现固定值控制，尚未连接到此配置项。

> 注意：路径均相对于项目根目录解析。

## 脚本说明

### 脚本分段与文件格式

脚本使用 `#数字` 作为段落分隔标记，例如 `#1`、`#2`。引擎按段读取剧情内容。
推荐把运行入口写成 `*.fss`。若你只维护 `*.fs`（`#` 后不带数字），可以通过 `preprogress_script` 在启动时自动补号并生成 `*.fss`。

### 脚本读取与执行机制

1. **启动预处理（可选）**
   - `Game::new()` 会遍历 `setting.preprogress_script`。
   - 每个源脚本会被处理为 `resource/<同名>.fss`，规则是把形如 `#` 的段标记自动补成 `#1/#2/...`。
2. **进入剧情时初始化脚本环境**
   - 创建 `ScriptContext`，注册全局变量、全局函数、类型（如 `character`、`text_obj`）和行为映射。
   - 入口脚本来自 `setting.entre_script`。
3. **按段流式读取**
   - `StreamSectionReader` 每次按 `session_id` 读取一段（从 `#N` 到下一段标记前）。
   - 到文件末尾会返回 EOF，剧情结束后回主菜单。
4. **解析成命令**
   - `ScriptParser`（词法+语法）将段文本转换为命令序列（`set/once/wait/call/assignment/chain`）。
5. **解释执行（逐帧）**
   - `Interpreter` 将本段命令注入 `SessionExecutor`。
   - 每帧 `update()` 执行；遇到 `wait` 会进入等待态（时间等待或输入等待）。
   - 段结束会触发 `once` 回滚：本段通过 `once` 改过的值会恢复，避免跨段污染。

### 脚本环境：对象、函数与方法

运行时可用对象/常量会在启动时输出到项目根目录的 `script_env.txt`，用于查阅与调试。

全局对象（核心）：

- `bg`：背景状态（图片、黑边开关）。
- `bgm`：背景音乐状态。
- `env_effect`：环境音状态。
- `frame`：对话框状态（显示、内容、打字机参数等）。
- `paragraph`：旁白/大段文本区状态。
- `chapter`：章节标题/副标题状态。
- `character_ls`：当前显示角色列表。
- `layers`：动态图层表（可增删）。

全局函数（常用）：

- `text(content)`：写入 `frame.content` 并直接驱动 `FrameBehaviour`（旁白模式：无说话人、无头像）。
- `voice(path, [seconds], [volume])`：播放语音；`seconds`>0 时先淡出再淡入，`volume` 为 0~1 音源音量系数；传空字符串可停止语音轨（支持淡出）。
- `add_layer(type, [name], source)`：添加动态图层。
- `del_layer(name)`：删除动态图层。
- `see(name)`：打印指定可视元素当前信息（调试）。
- `log(path_or_expr)`：打印某个脚本路径值（调试）。
- `save_to(table, target_path)`：将脚本表序列化到文件。
- `create_default_character(path)`：生成默认角色配置文件模板。

对象方法（常用）：

- `bg.set(path)` / `bg.trans_to(path, duration)` / `bg.show_edge()` / `bg.hide_edge()`
  - `bg.set("")` 支持清空背景图，此时会按主题色稳定填充背景区域。
- `bgm.set(path, [fade_type], [seconds], [volume])` / `bgm.stop([seconds])`
  - `fade_type` 可直接用环境里注册的全局字符串（如 `FADE_IN` / `FADE_OUT` / `TRANSITION`）。
  - 传入 `seconds` 时按秒控制淡入淡出时长（例如：`bgm.set "resource/bgm/abaddons_abyss.ogg" FADE_IN 4`）。
  - `volume` 为该次 `bgm.set` 音源音量系数（0~1，默认 1），与 track 音量相乘生效。
  - `bgm.stop 2` 会先淡出约 2 秒后再停止；不传参数时立即停止。
- `env_effect.set(path, [seconds], [volume])` / `env_effect.stop([seconds])`
  - `seconds`>0 时先淡出当前环境音后再淡入新环境音。
  - `volume` 为该次环境音音源音量系数（0~1），与 track 音量相乘生效。
- `frame.show()` / `frame.hide()` / `frame.set_mode(mode)`
- `paragraph.show()` / `paragraph.hide()` / `paragraph.print(text)` / `paragraph.new(text)` / `paragraph.clear()`
- `chapter.show_title(title, [duration])` / `chapter.show_sub_title(subtitle, [duration])`
  - 传空字符串可隐藏对应标题（例如 `chapter.show_title "" 0.2`）。
- `character.say(text)`（`character` 实例方法）
- `character_ls.set_characters(c1, c2, ...)`

### 常见命令语法

根据当前解析器，脚本支持以下形式：

- `变量 = 值`（赋值）
- `变量 = 命令 参数...`（命令返回值赋值）
- `对象.方法 参数...`（调用）
- `set 路径 参数...`（设置）
- `once 路径 参数...`（一次性命令,在该段落结束后会还原）
- `wait 0.5`（等待时间）
- `命令1 -> 命令2`（链式调用）

### 最小示例

<a href="https://github.com/rabitank/TermAVG/blob/main/resource/script_example.fs">查看示例脚本</a>

另外,运行后将生成`script_env.txt`文件打印所有脚本环境可用的对象和方法,可以辅助调试和写脚本.

## layout 布局说明

`layout.toml` 决定了剧情页、主菜单和弹窗在逻辑坐标系中的布局。

### 坐标与尺寸约定

- `ltwh`：`(left, top, width, height)`。
- `twh`：`(top, width, height)`（横向默认居中，由角色数量和间距推导 x）。
- `lw`：`(left, width)`。
- `wh`：`(width, height)`。

### layout.toml 字段含义

- `character_twh`：角色立绘框 `(top, width, height)`。
- `two_character_spec`：2 人同屏时间距。
- `x_character_spec`：3 人及以上同屏时间距。
- `vertical_dark_edge`：背景上下黑边高度。
- `frame_face_ltwh`：头像框区域。
- `frame_content_ltwh`：对话框主体区域。
- `text_ltwh`：对话正文文本区域（在 `frame_content` 内裁剪）。
- `frame_name_ltwh`：说话人名字区域。
- `short_key_ltwh`：底部快捷键提示条区域。
- `chapter_title_ltwh`：章节标题区域。
- `chapter_subtitle_ltwh`：章节副标题区域。
- `paragraph_ltwh`：旁白/段落文本框区域。
- `history_wh`：历史记录弹窗宽高。
- `mainmenu_lw`：主菜单列表面板的 `(left, width)`。
- `mainmenu_load_pop_lw`：主菜单下 Load 弹窗的 `(left, width)`；当 `width = 0` 时使用剩余宽度。

## 开发说明

- 工作区包含多个 crate，建议在项目根目录执行 `cargo check` / `cargo test`。
- 与脚本相关的核心代码位于 `tmj_core/src/script/`。
- 引擎页面和渲染流程位于 `tmj_app/src/pages/` 与 `tmj_app/src/game.rs`。

### 快捷键规范

所有界面统一使用以下快捷键：

| 按键 | 功能 |
|------|------|
| `↑/↓` 或 `←/→` | 方向移动 / 选择 |
| `Enter` | 确认 / 继续 |
| `Esc` 或 `q` | 返回 / 退出 |

各弹窗和页面底部均有快捷键提示栏。

## 依赖

- TUI: [ratatui](https://github.com/ratatui/ratatui), [crossterm](https://github.com/crossterm-rs/crossterm)
- GPU 渲染: [eframe](https://github.com/emilk/egui), [soft_ratatui](https://github.com/gold-silver-copper/soft_ratatui)
- 序列化: [serde](https://github.com/serde-rs/serde), [toml](https://github.com/toml-rs/toml)
- 音频: [rodio](https://github.com/RustAudio/rodio)
- 其他: `tracing`, `anyhow`, `image`, `strum`, `fontdue`, `cosmic-text`

## 贡献

目前添加功能,完善示例中,欢迎提交 Issue 和 PR

<!-- links -->
[contributors-shield]: https://img.shields.io/github/contributors/rabitank/TermAVG.svg?style=flat-square
[contributors-url]: https://github.com/rabitank/TermAVG/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/rabitank/TermAVG.svg?style=flat-square
[forks-url]: https://github.com/rabitank/TermAVG/network/members
[stars-shield]: https://img.shields.io/github/stars/rabitank/TermAVG.svg?style=flat-square
[stars-url]: https://github.com/rabitank/TermAVG/stargazers
[issues-shield]: https://img.shields.io/github/issues/rabitank/TermAVG.svg?style=flat-square
[issues-url]: https://github.com/rabitank/TermAVG/issues
