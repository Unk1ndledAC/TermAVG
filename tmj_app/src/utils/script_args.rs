use std::time::Duration;

use anyhow::anyhow;
use tmj_core::script::{ContextRef, ScriptValue, TabelGet, Table};

pub fn parse_arg<T, F>(args: &[ScriptValue], index: usize, default: T, parse: F) -> T
where
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    match args.get(index) {
        Some(raw) => match parse(raw) {
            Some(parsed) => parsed,
            None => {
                tracing::warn!(
                    "script arg[{index}] parse failed, value={:?}, fallback to default",
                    raw
                );
                default
            }
        },
        None => {
            tracing::warn!("script arg[{index}] missing, fallback to default");
            default
        }
    }
}

pub fn parse_required_arg<T, F>(
    args: &[ScriptValue],
    index: usize,
    parse: F,
) -> anyhow::Result<T>
where
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    let raw = args.get(index).ok_or_else(|| {
        let err = format!("missing required arg at index {index}");
        tracing::warn!("{err}");
        anyhow!(err)
    })?;

    parse(raw).ok_or_else(|| {
        let err = format!("failed to parse required arg[{index}], raw={:?}", raw);
        tracing::warn!("{err}");
        anyhow!(err)
    })
}

pub fn parse_duration(args: &[ScriptValue], index: usize, default_secs: f64) -> Duration {
    Duration::from_secs_f64(parse_arg(args, index, default_secs, ScriptValue::to_number).max(0.0))
}

pub fn parse_volume(args: &[ScriptValue], index: usize, default: f64) -> f32 {
    parse_arg(args, index, default, ScriptValue::to_number).clamp(0.0, 1.0) as f32
}

/// 从 table 取成员并转型；缺失或解析失败时回退 `default`（语义同 [`parse_arg`]）。
pub fn parse_member<T, G, F>(
    table: &G,
    key: impl ToString,
    default: T,
    parse: F,
) -> T
where
    G: TabelGet,
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    let key = key.to_string();
    match table.get(&key) {
        Ok(raw) => match parse(&raw) {
            Some(parsed) => parsed,
            None => {
                tracing::warn!(
                    "table[{key}] parse failed, value={raw:?}, fallback to default"
                );
                default
            }
        },
        Err(e) => {
            tracing::warn!("table[{key}] missing ({e}), fallback to default");
            default
        }
    }
}

/// 从 table 取必填成员并转型；缺失或解析失败时返回错误（语义同 [`parse_required_arg`]）。
pub fn parse_required_member<T, G, F>(
    table: &G,
    key: impl ToString,
    parse: F,
) -> anyhow::Result<T>
where
    G: TabelGet,
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    let key = key.to_string();
    let raw = table.get(&key).map_err(|e| {
        let err = format!("missing required table member [{key}]: {e}");
        tracing::warn!("{err}");
        anyhow!(err)
    })?;

    parse(&raw).ok_or_else(|| {
        let err = format!("failed to parse table[{key}], raw={raw:?}");
        tracing::warn!("{err}");
        anyhow!(err)
    })
}

/// 从 `Table` 取字段（支持点路径）；`ctx` 在路径含 `TableHandle` 时必填。
pub fn parse_table_field<T, F>(
    table: &Table,
    key: &str,
    ctx: Option<&ContextRef>,
    default: T,
    parse: F,
) -> T
where
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    match table.get(key, ctx) {
        Some(raw) => match parse(&raw) {
            Some(parsed) => parsed,
            None => {
                tracing::warn!(
                    "table field[{key}] parse failed, value={raw:?}, fallback to default"
                );
                default
            }
        },
        None => {
            tracing::warn!("table field[{key}] missing, fallback to default");
            default
        }
    }
}

/// 从 `Table` 取必填字段（支持点路径）。
pub fn parse_required_table_field<T, F>(
    table: &Table,
    key: &str,
    ctx: Option<&ContextRef>,
    parse: F,
) -> anyhow::Result<T>
where
    F: FnOnce(&ScriptValue) -> Option<T>,
{
    let raw = table.get(key, ctx).ok_or_else(|| {
        let err = format!("missing required table field [{key}]");
        tracing::warn!("{err}");
        anyhow!(err)
    })?;

    parse(&raw).ok_or_else(|| {
        let err = format!("failed to parse table field[{key}], raw={raw:?}");
        tracing::warn!("{err}");
        anyhow!(err)
    })
}
