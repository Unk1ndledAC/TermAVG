// src/script/command_executor.rs
use crate::script::{
    Command, CommandBlockType, ContextRef, ScriptContext, ScriptValue, WaitCondition,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum ExecuteStatus {
    Completed,
    Waiting(WaitCondition),
    Error(String),
}

pub struct CommandExecutor {
    command: Command,
    block_type: CommandBlockType,
    state: ExecutorState,
    /// 时间等待的剩余时间
    time_remaining: Option<f64>,
    /// Chain：下一条待执行命令下标
    chain_index: usize,
    /// Chain：当前子命令（含 wait 恢复状态）
    chain_sub: Option<Box<CommandExecutor>>,
}

enum ExecutorState {
    Ready,
    Waiting(WaitCondition),
    Completed,
    Error(String),
}

impl CommandExecutor {
    pub fn new(command: Command) -> Self {
        let block_type = command.block_type();
        CommandExecutor {
            command,
            block_type,
            state: ExecutorState::Ready,
            time_remaining: None,
            chain_index: 0,
            chain_sub: None,
        }
    }

    pub fn step(&mut self, context: &ContextRef) -> ExecuteStatus {
        match &self.state {
            ExecutorState::Ready => self.execute(context),
            ExecutorState::Waiting(condition) => ExecuteStatus::Waiting(condition.clone()),
            ExecutorState::Completed => ExecuteStatus::Completed,
            ExecutorState::Error(e) => ExecuteStatus::Error(e.clone()),
        }
    }

    /// 更新时间等待
    pub fn update(&mut self, delta_time: f64) -> bool {
        if let Some(sub) = self.chain_sub.as_mut() {
            return sub.update(delta_time);
        }

        if let ExecutorState::Waiting(WaitCondition::Time(total)) = &self.state {
            let remaining = self.time_remaining.get_or_insert(*total);
            *remaining -= delta_time;

            if *remaining <= 0.0 {
                self.state = ExecutorState::Completed;
                return true;
            }
        }
        false
    }

    /// 处理事件
    pub fn handle_event(&mut self, event: &InputEvent) -> bool {
        if let Some(sub) = self.chain_sub.as_mut() {
            return sub.handle_event(event);
        }

        if let ExecutorState::Waiting(condition) = &self.state {
            let should_resume = match condition {
                WaitCondition::Click => matches!(event, InputEvent::Click),
                WaitCondition::Input(expected) => {
                    matches!(event, InputEvent::Text(text) if text == expected)
                }
                WaitCondition::Any(conditions) => conditions.iter().any(|c| match c {
                    WaitCondition::Click => matches!(event, InputEvent::Click),
                    WaitCondition::Input(expected) => {
                        matches!(event, InputEvent::Text(text) if text == expected)
                    }
                    _ => false,
                }),
                _ => false,
            };

            if should_resume {
                self.state = ExecutorState::Completed;
                return true;
            }
        }
        false
    }

    fn is_chain(&self) -> bool {
        matches!(self.command, Command::Chain { .. })
    }

    pub fn is_waiting(&self) -> bool {
        if let Some(sub) = self.chain_sub.as_ref() {
            return sub.is_waiting();
        }
        matches!(self.state, ExecutorState::Waiting(_))
    }

    pub fn wait_condition(&self) -> Option<WaitCondition> {
        match &self.state {
            ExecutorState::Waiting(c) => Some(c.clone()),
            _ => None,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state, ExecutorState::Completed)
    }

    pub fn is_blocking(&self) -> bool {
        self.block_type == CommandBlockType::Blocking
    }

    /// 获取剩余等待时间 (用于外部计时器)
    pub fn remaining_time(&self) -> Option<f64> {
        if let Some(sub) = self.chain_sub.as_ref() {
            return sub.remaining_time();
        }
        if let ExecutorState::Waiting(WaitCondition::Time(_)) = &self.state {
            self.time_remaining
        } else {
            None
        }
    }

    /// 将当前等待压缩到至多 `buffer_secs`（时间等待取 min；点击/输入等待改为一帧时间等待）。
    pub fn skip_wait_with_buffer(&mut self, buffer_secs: f64) -> bool {
        if let Some(sub) = self.chain_sub.as_mut() {
            return sub.skip_wait_with_buffer(buffer_secs);
        }
        if !self.is_waiting() {
            return false;
        }
        let buffer = buffer_secs.max(0.0);
        match &self.state {
            ExecutorState::Waiting(WaitCondition::Time(_)) => {
                let remaining = self.time_remaining.get_or_insert(buffer);
                *remaining = (*remaining).min(buffer);
                self.state = ExecutorState::Waiting(WaitCondition::Time(*remaining));
            }
            ExecutorState::Waiting(_) => {
                self.time_remaining = Some(buffer);
                self.state = ExecutorState::Waiting(WaitCondition::Time(buffer));
            }
            _ => return false,
        }
        true
    }

    fn execute(&mut self, context: &Rc<RefCell<ScriptContext>>) -> ExecuteStatus {
        let result = if self.is_chain() {
            self.advance_chain(context)
        } else {
            self.execute_command(context)
        };

        match &result {
            ExecuteStatus::Waiting(WaitCondition::Time(total)) => {
                if self.chain_sub.is_none() {
                    self.time_remaining = Some(*total);
                    self.state = ExecutorState::Waiting(WaitCondition::Time(*total));
                }
            }
            ExecuteStatus::Waiting(condition) => {
                if self.chain_sub.is_none() {
                    self.state = ExecutorState::Waiting(condition.clone());
                }
            }
            ExecuteStatus::Completed => {
                self.state = ExecutorState::Completed;
            }
            ExecuteStatus::Error(e) => {
                self.state = ExecutorState::Error(e.clone());
            }
        }

        result
    }

    /// 逐条执行 Chain，保留 `chain_index` / `chain_sub`，wait 结束后不从头重跑。
    fn advance_chain(&mut self, context: &Rc<RefCell<ScriptContext>>) -> ExecuteStatus {
        let Command::Chain { commands } = &self.command else {
            return ExecuteStatus::Error("advance_chain called on non-Chain".to_string());
        };
        let commands = commands.as_slice();
        loop {
            if self.chain_index >= commands.len() {
                self.chain_sub = None;
                return ExecuteStatus::Completed;
            }

            if self.chain_sub.is_none() {
                self.chain_sub = Some(Box::new(CommandExecutor::new(
                    commands[self.chain_index].clone(),
                )));
            }

            let sub = self.chain_sub.as_mut().expect("chain_sub just set");
            match sub.step(context) {
                ExecuteStatus::Completed => {
                    self.chain_sub = None;
                    self.chain_index += 1;
                }
                ExecuteStatus::Waiting(condition) => {
                    return ExecuteStatus::Waiting(condition);
                }
                ExecuteStatus::Error(e) => {
                    self.chain_sub = None;
                    return ExecuteStatus::Error(e);
                }
            }
        }
    }

    fn execute_command(&mut self, context: &Rc<RefCell<ScriptContext>>) -> ExecuteStatus {
        match &self.command {
            Command::Assignment { name, value } => {
                let mut ctx = context.borrow_mut();
                ctx.set_global_val(name, value.clone());

                ExecuteStatus::Completed
            }
            Command::CommandAssignment {
                name,
                command,
                args,
            } => {
                let args = {
                    let ctx = context.borrow();
                    ctx.parse_args(args)
                };
                // 0. 如果 command 是已注册类型，先复制构建器函数，再执行构建，避免 RefCell 重入借用
                // 这里纯ai解决不了refcell的借用问题内联了build_type_instance函数,不过只调用一次因此随便了
                let type_builders = {
                    let ctx = context.borrow();
                    ctx.type_registry.get_type_builders(command)
                };
                if let Some((data_f, method_f)) = type_builders {
                    let mut table = data_f(&mut context.borrow_mut(), args.to_vec());
                    table.set_type_tag(command);
                    let table_rc = Rc::new(RefCell::new(table));
                    match method_f(context, &table_rc).and_then(|_| {
                        context
                            .borrow_mut()
                            .register_script_value_tables(&ScriptValue::Table(table_rc.clone()))?;
                        Ok(())
                    }) {
                        Ok(_) => {
                            context
                                .borrow_mut()
                                .set_global_val(name, ScriptValue::Table(table_rc));
                            ExecuteStatus::Completed
                        }
                        Err(s) => ExecuteStatus::Error(format!(
                            "assign {} failed: when buiding instane: {}",
                            name, s
                        )),
                    }
                // 1. 执行命令
                } else {
                    let result = self.execute_command_call(context, command, &args);

                    match result {
                        Ok(return_value) => {
                            // 2. 将返回值赋给变量
                            let mut ctx = context.borrow_mut();
                            ctx.set_global_val(name, return_value);
                            ExecuteStatus::Completed
                        }
                        Err(e) => ExecuteStatus::Error(e.to_string()),
                    }
                }
            }
            Command::Set { path, args } => {
                let args = {
                    let ctx = context.borrow();
                    ctx.parse_args(args)
                };
                self.execute_set(context, path, &args, false)
            }

            Command::Once { path, args } => {
                let args = {
                    let ctx = context.borrow();
                    ctx.parse_args(args)
                };

                self.execute_set(context, path, &args, true)
            }

            Command::Wait { condition } => ExecuteStatus::Waiting(condition.clone()),

            Command::Call { path, args } => {
                let args = {
                    let ctx = context.borrow();
                    ctx.parse_args(args)
                };

                match {context.borrow().resolve_path(path)} {
                    Ok(ScriptValue::Function(func)) => match func.call(&context, args.clone()) {
                        Ok(_) => ExecuteStatus::Completed,
                        Err(e) => ExecuteStatus::Error(e.to_string()),
                    },
                    Ok(val) => ExecuteStatus::Error(format!("{:?} is not a function", val)),
                    Err(e) => ExecuteStatus::Error(e),
                }
            }

            Command::Next { target } => {
                context.borrow_mut().set_next_session_target(*target);
                ExecuteStatus::Completed
            }

            Command::Chain { .. } => ExecuteStatus::Error(
                "Chain must be executed via advance_chain".to_string(),
            ),

            Command::Empty => ExecuteStatus::Completed,
        }
    }

    /// 执行命令调用并获取返回值
    fn execute_command_call(
        &self,
        context: &Rc<RefCell<ScriptContext>>,
        command: &str,
        args: &[ScriptValue],
    ) -> anyhow::Result<ScriptValue> {
        // 解析命令路径
        match context.borrow().resolve_path(command) {
            Ok(ScriptValue::Function(func)) => {
                // 调用函数，获取返回值
                func.call(&context, args.to_vec())
            }
            Ok(val) => {
                // 如果不是函数，返回对象本身 (如 UserData)
                Ok(val)
            }
            Err(_) => {
                // 路径不存在，可能是全局方法
                // 尝试从 globals 直接查找
                if let Some(func) = context.borrow().get_global_val(command) {
                    if let ScriptValue::Function(f) = func {
                        return f.call(&context, args.to_vec());
                    }
                }
                anyhow::bail!("Command '{}' not found", command)
            }
        }
    }

    fn execute_set(
        &self,
        context: &Rc<RefCell<ScriptContext>>,
        path: &str,
        args: &[ScriptValue],
        is_once: bool,
    ) -> ExecuteStatus {
        let mut ctx = context.borrow_mut();
        let parts: Vec<&str> = path.split('.').collect();

        if parts.len() == 1 {
            let old_value = ctx.get_global_val(path).unwrap_or(ScriptValue::nil());

            if args.is_empty() {
                return ExecuteStatus::Error("set requires at least one argument".to_string());
            }

            if is_once {
                ctx.push_once_record(crate::script::OnceRecord {
                    path: path.to_string(),
                    field: None,
                    old_value: old_value.clone(),
                });
            }

            ctx.set_global_val(path, args[0].clone());
            ExecuteStatus::Completed
        } else {
            let obj_name = parts[0];
            let field_path = parts[1..].join(".");

            let obj = match ctx.get_global_val(obj_name) {
                Some(val) => val,
                None => return ExecuteStatus::Error(format!("Global '{}' not found", obj_name)),
            };

            drop(ctx);

            match obj {
                ScriptValue::Table(_) | ScriptValue::TableHandle(_) => {
                    let table = match context.borrow().resolve_table_value(&obj) {
                        Ok(t) => t,
                        Err(e) => return ExecuteStatus::Error(e),
                    };
                    let ctx_opt = context.borrow().context_ref();
                    let old_value = table
                        .borrow()
                        .get(&field_path, ctx_opt.as_ref())
                        .unwrap_or(ScriptValue::nil());

                    if args.is_empty() {
                        return ExecuteStatus::Error(
                            "set requires at least one argument".to_string(),
                        );
                    }

                    table
                        .borrow_mut()
                        .set(&field_path, args[0].clone(), ctx_opt.as_ref());

                    if is_once {
                        let mut ctx = context.borrow_mut();
                        ctx.push_once_record(crate::script::OnceRecord {
                            path: path.to_string(),
                            field: Some(field_path),
                            old_value,
                        });
                    }

                    ExecuteStatus::Completed
                }
                _ => ExecuteStatus::Error(format!("Cannot set on {:?}", obj)),
            }
        }
    }
}

/// 输入事件
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Click,
    Text(String),
    Key(char),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Command, ScriptContext, WaitCondition};
    use std::{cell::RefCell, rc::Rc};

    fn test_ctx() -> ContextRef {
        Rc::new(RefCell::new(ScriptContext::new()))
    }

    #[test]
    fn chain_resumes_after_wait_without_restarting() {
        let chain = Command::Chain {
            commands: vec![
                Command::Wait {
                    condition: WaitCondition::Time(1.0),
                },
                Command::Wait {
                    condition: WaitCondition::Time(2.0),
                },
            ],
        };
        let mut ex = CommandExecutor::new(chain);
        let ctx = test_ctx();

        assert!(matches!(
            ex.step(&ctx),
            ExecuteStatus::Waiting(WaitCondition::Time(t)) if (t - 1.0).abs() < f64::EPSILON
        ));
        assert!(ex.chain_sub.is_some());

        assert!(ex.update(1.0));
        assert!(matches!(
            ex.step(&ctx),
            ExecuteStatus::Waiting(WaitCondition::Time(t)) if (t - 2.0).abs() < f64::EPSILON
        ));

        assert!(ex.update(2.0));
        assert!(matches!(ex.step(&ctx), ExecuteStatus::Completed));
    }

    #[test]
    fn standalone_wait_does_not_repeat_after_time_elapsed() {
        let mut ex = CommandExecutor::new(Command::Wait {
            condition: WaitCondition::Time(0.5),
        });
        let ctx = test_ctx();

        assert!(matches!(
            ex.step(&ctx),
            ExecuteStatus::Waiting(WaitCondition::Time(_))
        ));
        assert!(ex.update(0.5));
        assert!(matches!(ex.step(&ctx), ExecuteStatus::Completed));
    }
}
