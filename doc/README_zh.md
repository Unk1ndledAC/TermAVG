# TermAVG (TMJ)

<p align="center">
  <img src="doc/logo.png" alt="TMJ Logo" width="200" height="80">
</p>

<p align="center">
  一个使用 Rust 编写的终端像素风文字冒险（AVG）引擎。<br />
  基于脚本解释器驱动剧情，支持对话、角色、音频与存档流程。<br />
  <a href="https://github.com/rabitank/TerminalLove">查看示例游戏《终末之爱》</a>
</p>

## 特性

<p align="center">
  <img src="doc/example.png" alt="Windows Terminal 截图" width="480" height="320">
</p>

- **双运行模式**：终端 TUI 模式（`tmj_terminal`）+ GPU 加速窗口模式（`tmj_egui`）
- **终端渲染**：基于 `ratatui` + `crossterm` 的 TUI 绘制与事件处理
- **GPU 渲染**：基于 `eframe` + `soft_ratatui`，支持全屏、1:2 字符网格与 CJK 字体
- **脚本驱动**：内置脚本解析器，支持赋值、调用、`wait`、链式调用等语法
- **模块化工作区**：`tmj_app`、`tmj_core`、`tmj_macro`、`tmj_terminal`、`tmj_egui`
- **可配置启动**：通过 `setting.toml` 指定分辨率、字体、资源路径与布局参数
- **音频与资源**：支持角色立绘、表情资源与音频播放

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

终端 TUI 模式（日志重定向避免界面冲突）：

```bash
cargo run -p tmj_terminal 2> debug.txt
```

GPU 窗口模式（自动全屏，支持 CJK 字体）：

```bash
cargo run -p tmj_egui
```

> 首次运行时若缺少 `setting.toml`，程序将按默认配置自动创建。

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

## 配置说明

### setting.toml

关键字段示例：

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

| 字段 | 说明 |
|------|------|
| `resolution` | 逻辑渲染分辨率 `[w, h]`（字符格单位），主画面依此居中绘制 |
| `font` | GPU 模式字体路径（推荐等宽 CJK 字体，如 Sarasa Term CL） |
| `font_bold` | GPU 模式粗体字体路径（可选） |
| `preprogress_script` | 需预处理的脚本列表，启动时 `*.fs` → 带段号的 `*.fss` |
| `is_force_skipable` | 预留字段，暂未使用 |
| `save_dir` | 存档目录（普通槽位与 `temp.save`） |
| `gallery_dir` | 图鉴资源目录 |
| `about_file` | 主菜单 About 弹窗内容文件（每行居中显示） |
| `entre_script` | 入口脚本路径（通常指向预处理后的 `*.fss`） |
| `mainmenu_title_file` | 主菜单标题文件（可选，缺省用默认标题） |
| `mainmenu_default_bg_img` | 主菜单默认背景图 |
| `mainmenu_session_bg_map` | 按 `session_id` 区间匹配背景图的映射列表：`session_id_min`、`session_id_max`、`bg_img` |
| `default_bg_img` / `default_face_img` | 兼容字段，当前主流程未直接读取 |
| `max_history_ls` | 兼容字段，历史上限由内部实现固定值控制 |

> 路径均相对于项目根目录解析。

### layout.toml

`layout.toml` 定义剧情页、主菜单与弹窗在逻辑坐标系中的布局。

坐标简写：
- `ltwh`：`(left, top, width, height)`
- `twh`：`(top, width, height)`（横向居中，由角色数与间距推导 x）
- `lw`：`(left, width)`
- `wh`：`(width, height)`

| 字段 | 含义 |
|------|------|
| `character_twh` | 角色立绘框 `(top, width, height)` |
| `two_character_spec` | 2 人同屏时间距 |
| `x_character_spec` | 3 人及以上同屏时间距 |
| `vertical_dark_edge` | 背景上下黑边高度 |
| `frame_face_ltwh` | 头像框区域 |
| `frame_content_ltwh` | 对话框主体区域 |
| `text_ltwh` | 正文文本区域（在 `frame_content` 内裁剪） |
| `frame_name_ltwh` | 说话人名字区域 |
| `short_key_ltwh` | 底部快捷键提示条区域 |
| `chapter_title_ltwh` | 章节标题区域 |
| `chapter_subtitle_ltwh` | 章节副标题区域 |
| `paragraph_ltwh` | 旁白 / 段落文本框区域 |
| `history_wh` | 历史记录弹窗宽高 |
| `mainmenu_lw` | 主菜单列表面板 `(left, width)` |
| `mainmenu_load_pop_lw` | Load 弹窗 `(left, width)`；`width = 0` 时使用剩余宽度 |

## 脚本系统

### 文件格式

脚本使用 `#数字` 作为段落分隔标记（如 `#1`、`#2`），引擎按段读取剧情。运行入口建议使用 `*.fss` 格式；若维护 `*.fs`（`#` 后无数字），可通过 `preprogress_script` 在启动时自动补号生成 `*.fss`。

### 执行流程

1. **启动预处理（可选）**：`Game::new()` 遍历 `preprogress_script`，将 `#` 标记补为 `#1`/`#2`/...，输出至 `resource/<同名>.fss`。
2. **初始化脚本环境**：创建 `ScriptContext`，注册全局变量、函数、类型（如 `character`、`text_obj`）与行为映射，入口为 `entre_script`。
3. **按段流式读取**：`StreamSectionReader` 按 `session_id` 读取一段（`#N` 至下一标记），文件末尾返回 EOF，剧情结束回主菜单。
4. **解析命令**：`ScriptParser`（词法+语法）将段文本转为命令序列（`set` / `once` / `wait` / `call` / `assignment` / `chain`）。
5. **逐帧解释执行**：`Interpreter` 将命令注入 `SessionExecutor`，每帧 `update()` 执行。遇到 `wait` 进入等待态（时间或输入），段结束时 `once` 修改的值自动回滚。

### 全局对象

| 对象 | 说明 |
|------|------|
| `bg` | 背景状态（图片、黑边开关） |
| `bgm` | 背景音乐状态 |
| `env_effect` | 环境音状态 |
| `frame` | 对话框状态（显示、内容、打字机参数等） |
| `paragraph` | 旁白 / 大段文本区状态 |
| `chapter` | 章节标题 / 副标题状态 |
| `character_ls` | 当前显示角色列表 |
| `layers` | 动态图层表（可增删） |

### 类型

| 类型 | 说明 |
|------|------|
| `character` | 角色对象，可通过 `character.say(text)` 驱动对话框说话，拥有立绘、表情、位置等属性 |
| `layer` | 动态图层对象，可通过 `layers` 全局对象管理增删，支持多种视觉效果（Alpha、故障、心跳等） |

### 全局函数

| 函数 | 说明 |
|------|------|
| `text(content)` | 写入 `frame.content`，驱动 `FrameBehaviour`（旁白模式，无说话人、无头像） |
| `voice(path, [seconds], [volume])` | 播放语音；`seconds>0` 时先淡出再淡入；`volume: 0~1`；传空字符串停止语音轨 |
| `see(name)` | 打印可视元素当前信息（调试） |
| `log(path_or_expr)` | 打印脚本路径值（调试） |
| `save_to(table, target_path)` | 将脚本表序列化到文件 |
| `create_default_character(path)` | 生成默认角色配置模板 |

### 常用对象方法

- `bg.set(path)` / `bg.trans_to(path, duration)` / `bg.show_edge()` / `bg.hide_edge()`
- `bgm.set(path, [fade_type], [seconds], [volume])` / `bgm.stop([seconds])`
- `env_effect.set(path, [seconds], [volume])` / `env_effect.stop([seconds])`
- `frame.show()` / `frame.hide()` / `frame.set_mode(mode)`
- `paragraph.show()` / `paragraph.hide()` / `paragraph.print(text)` / `paragraph.new(text)` / `paragraph.clear()`
- `chapter.show_title(title, [duration])` / `chapter.show_sub_title(subtitle, [duration])`
- `character.say(text)`（实例方法）
- `character_ls.set_characters(c1, c2, ...)`

### 命令语法

| 语法 | 说明 |
|------|------|
| `变量 = 值` | 赋值 |
| `变量 = 命令 参数...` | 命令返回值赋值 |
| `对象.方法 参数...` | 调用 |
| `set 路径 参数...` | 设置 |
| `once 路径 参数...` | 一次性命令，段落结束后自动还原 |
| `wait 0.5` | 等待指定时间（秒） |
| `命令1 -> 命令2` | 链式调用 |

> 运行时环境信息会在启动时输出至 `script_env.txt`，可用于调试与查阅。
> 参见[示例脚本](https://github.com/rabitank/TermAVG/blob/main/resource/script_example.fs)。

## 开发

- 工作区含多个 crate，建议在项目根执行 `cargo check` / `cargo test`
- 脚本核心代码位于 `tmj_core/src/script/`
- 引擎页面与渲染流程位于 `tmj_app/src/pages/` 及 `tmj_app/src/game.rs`

### 快捷键

| 按键 | 功能 |
|------|------|
| `↑/↓` 或 `←/→` | 移动 / 选择 |
| `Enter` | 确认 / 继续 |
| `Esc` 或 `q` | 返回 / 退出 |

各页面底部均有快捷键提示栏。

## 依赖

- **TUI**： [ratatui](https://github.com/ratatui/ratatui), [crossterm](https://github.com/crossterm-rs/crossterm)
- **GPU 渲染**： [eframe](https://github.com/emilk/egui), [soft_ratatui](https://github.com/gold-silver-copper/soft_ratatui)
- **序列化**： [serde](https://github.com/serde-rs/serde), [toml](https://github.com/toml-rs/toml)
- **音频**： [rodio](https://github.com/RustAudio/rodio)
- **其他**： `tracing`, `anyhow`, `image`, `strum`, `fontdue`, `cosmic-text`

## 贡献

欢迎提交 Issue 与 PR。

## 许可

本项目基于 [MIT 许可](LICENSE) 开源。  
Copyright (c) 2024 rabitank
