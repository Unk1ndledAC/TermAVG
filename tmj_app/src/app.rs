use anyhow::Context;
use anyhow::Result;
use ratatui::Terminal;
use ratatui::prelude::Backend;
use std::cell::RefCell;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tmj_core::command::CmdBuffer;
use tmj_core::event::EventManager;

use tmj_core::event::{GameEvent, handler::EventDispatcher};

use crate::game::Game;

pub struct App<T: Backend> {
    pub terminal: Terminal<T>,
    pub game: RefCell<Game>,
}

impl<T: Backend> App<T> {
    pub fn new(terminal: Terminal<T>) -> Self
    where
        T: Backend,
    {
        let game: RefCell<Game> = Game::new().into();
        App { terminal, game }
    }

    pub fn main_loop<F>(
        app: &mut App<T>,
        receiver: &Receiver<GameEvent>,
        tick_rate: Duration,
        event_forward: F,
    ) -> Result<()>
    where
        F: Fn() -> () + 'static,
    {
        let mut last_tick = std::time::Instant::now();
        let mut game = app.game.borrow_mut();
        EventManager::with_looper(|l| {
            l.cool_down(Duration::from_millis(100));
        });
        loop {
            let last_tick_time = last_tick.elapsed();
            last_tick = std::time::Instant::now();

            // 事件冷静, 即屏蔽事件接收一段时间
            EventManager::with_looper(|l| {
                if !l.check_is_warmup() {
                    l.drain_buffer(receiver);
                }
            });
            event_forward();

            if let Ok(event) = receiver.try_recv() {
                if !game
                    .handle_event(&event)
                    .context("app handle event failed!")?
                {
                    return Ok(());
                }
            }

            game.handle_tick(last_tick_time);

            for cmd in CmdBuffer::take_commands() {
                game.handle_cmd(&cmd)
                    .context(format!("game handle cmd:{} failed!", cmd))?;
            }
            app.terminal.draw(|f| game.draw(f));

            thread::sleep(tick_rate.saturating_sub(last_tick.elapsed()));

            if game.game_flow.borrow().is_ready_quit() {
                break Ok(());
            }
        }
    }
}
