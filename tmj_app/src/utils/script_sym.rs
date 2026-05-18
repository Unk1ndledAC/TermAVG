//! 脚本 API 符号登记表（由 `script_sym!` 通过 inventory 收集）。

/// `script_sym!` 第二参数：符号在脚本对象中的角色分类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptSymCategory {
    /// 全局表 / 可构造类型名（如 `bg`、`layer`）
    Type,
    /// 表字段、全局配置项（如 `m_visible`、`bgimg_path`）
    Member,
    /// 可调用方法（如 `show`、`set`）
    Function,
}

impl ScriptSymCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Member => "member",
            Self::Function => "function",
        }
    }
}

/// 单条 `script_sym!` 登记项。
pub struct ScriptSymEntry {
    pub const_name: &'static str,
    pub value: &'static str,
    pub category: ScriptSymCategory,
    pub description: &'static str,
    pub module: &'static str,
}

inventory::collect!(ScriptSymEntry);

/// 将登记项写入 `script_env.txt`：按模块分组，组内按 type → member → function 排序。
pub fn write_script_sym_reference(path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;

    let mut entries: Vec<_> = inventory::iter::<ScriptSymEntry>.into_iter().collect();
    entries.sort_by(|a, b| {
        a.module
            .cmp(b.module)
            .then(a.category.cmp(&b.category))
            .then(a.value.cmp(b.value))
    });

    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "# 脚本 API 符号参考（由 script_sym! 宏登记，启动时自动生成）"
    )?;
    writeln!(
        file,
        "# 格式: [分类] 常量名 → 脚本键    # 说明"
    )?;
    writeln!(file)?;

    let mut current_module = "";
    for e in &entries {
        if e.module != current_module {
            if !current_module.is_empty() {
                writeln!(file)?;
            }
            writeln!(file, "## {}", e.module)?;
            writeln!(
                file,
                "  {:8} {:18} → {:18}  # {}",
                "分类",
                "常量名",
                "脚本键",
                "说明"
            )?;
            writeln!(file, "  {}", "─".repeat(72))?;
            current_module = e.module;
        }
        writeln!(
            file,
            "  {:8} {:18} → {:18}  # {}",
            e.category.label(),
            e.const_name,
            e.value,
            e.description
        )?;
    }

    writeln!(file)?;
    Ok(())
}
