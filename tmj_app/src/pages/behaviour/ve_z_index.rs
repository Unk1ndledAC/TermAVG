//! 对话场景各 Behaviour 创建的 VisualElement 的 z_index 常量表。
//!
//! 启动时由 `Game::new` 写入 `ve_z_index.txt`（见 [`write_ve_z_index_reference`]）。

/// 供 `ve_z_index!` 宏通过 inventory 收集，用于生成参考文件。
pub struct VeZIndexEntry {
    pub name: &'static str,
    pub value: i32,
    pub description: &'static str,
}

inventory::collect!(VeZIndexEntry);

use tmj_core::script::ve_z_index;

// --- background ---
ve_z_index!(Z_BG, 1000, "背景主图（含过渡绘制）");
ve_z_index!(Z_BG_EDGE, 1500, "画面上/下黑边遮罩");

// --- chapter ---
ve_z_index!(Z_CHAPTER_TITLE, 2100, "章节标题");
ve_z_index!(Z_CHAPTER_SUBTITLE, 2200, "章节副标题");

// --- character ---
ve_z_index!(Z_CHARACTER_BASE, 3000, "立绘层基准，运行时 z_index = Z_CHARACTER_BASE + ls_id");

// --- dialogue frame ---
ve_z_index!(Z_FRAME_BLOCK, 4000, "对话框底板");
ve_z_index!(Z_FRAME_TEXT, 4100, "对话框正文");
ve_z_index!(Z_FRAME_NAME, 4200, "说话人名称");
ve_z_index!(Z_FRAME_SHORTKEY, 4300, "快捷键提示栏");
ve_z_index!(Z_FRAME_FACE, 4400, "对话框内头像");

// --- paragraph ---
ve_z_index!(Z_PARAGRAPH_TEXT, 5000, "旁白/段落文本框");

/// 将登记过的 z_index 常量写入 `ve_z_index.txt`（按数值升序）。
pub fn write_ve_z_index_reference(path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;

    let mut entries: Vec<_> = inventory::iter::<VeZIndexEntry>.into_iter().collect();
    entries.sort_by(|a, b| a.value.cmp(&b.value).then_with(|| a.name.cmp(b.name)));

    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "# VisualElement z_index 参考（由 ve_z_index! 宏登记，启动时自动生成）"
    )?;
    writeln!(file, "# 格式: 常量名 = 值  # 说明")?;
    writeln!(file)?;

    for e in entries {
        writeln!(file, "{:24} = {:4}  # {}", e.name, e.value, e.description)?;
    }

    writeln!(file)?;
    writeln!(
        file,
        "# 未在此表中的动态层：layer.* 由脚本 layer.z_deep 指定；character_* 见 Z_CHARACTER_BASE"
    )?;

    Ok(())
}
