use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

use crate::event::GameEvent;
use crate::event::provider::{CrosstermProvider, EventProvider};

pub struct EventLooper {
    pub sender: SyncSender<GameEvent>,
    close_flag: Arc<AtomicBool>,
    _thread_handle: Option<thread::JoinHandle<()>>,
    start_time: Instant,
    warmup_duration: Duration,
}

impl EventLooper {
    pub fn new(buffer_size: usize, poll_timeout: u64) -> (Self, Receiver<GameEvent>) {
        Self::new_with_provider(
            buffer_size,
            Box::new(CrosstermProvider::new(Duration::from_millis(poll_timeout))),
        )
    }

    pub fn new_with_provider(
        buffer_size: usize,
        provider: Box<dyn EventProvider>,
    ) -> (Self, Receiver<GameEvent>) {
        let (sender, reciver) = mpsc::sync_channel(buffer_size);
        let close_flag = Arc::new(AtomicBool::new(false));
        let close_flag_cloned = close_flag.clone();
        let sender_cloned = sender.clone();

        let thread_handle = thread::spawn(move || {
            let mut provider = provider;
            loop {
                if close_flag_cloned.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some(event) = provider.poll_event() {
                    if sender_cloned.send(event).is_err() {
                        break;
                    }
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        (
            Self {
                sender,
                close_flag,
                _thread_handle: Some(thread_handle),
                start_time: Instant::now(),
                warmup_duration: Duration::from_millis(500),
            },
            reciver,
        )
    }

    pub fn drain_buffer(&self, receiver: &Receiver<GameEvent>) {
        while receiver.try_recv().is_ok() {}
    }

    pub fn check_is_warmup(&self) -> bool {
        self.start_time.elapsed() > self.warmup_duration
    }

    pub fn cool_down(&mut self, duration: Duration) {
        self.start_time = Instant::now();
        self.warmup_duration = duration;
    }

    pub fn stop(&self) {
        self.close_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
