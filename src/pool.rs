use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

const MAX_THREADS: usize = 8;
static POOL: OnceLock<Pool> = OnceLock::new();

pub fn pool() -> &'static Pool {
    POOL.get_or_init(|| {
        let threads = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1);
        Pool::new(threads.min(MAX_THREADS) - 1)
    })
}

struct Job(*const (dyn Fn() + Sync));

unsafe impl Send for Job {}

struct Countdown {
    remaining: Mutex<usize>,
    finished: Condvar,
}

struct Ticket<'a>(&'a Countdown);

impl Drop for Ticket<'_> {
    fn drop(&mut self) {
        let mut remaining = self.0.remaining.lock().unwrap();
        *remaining -= 1;
        if *remaining == 0 {
            self.0.finished.notify_all();
        }
    }
}

pub struct Pool {
    senders: Vec<Sender<Job>>,
    countdown: Arc<Countdown>,
}

impl Pool {
    fn new(workers: usize) -> Self {
        let countdown = Arc::new(Countdown {
            remaining: Mutex::new(0),
            finished: Condvar::new(),
        });
        let mut senders = Vec::with_capacity(workers);

        for _ in 0..workers {
            let (sender, receiver) = channel::<Job>();
            let countdown = countdown.clone();
            std::thread::spawn(move || {
                while let Ok(job) = receiver.recv() {
                    let _ticket = Ticket(&countdown);
                    unsafe { (*job.0)() };
                }
            });
            senders.push(sender);
        }

        Self { senders, countdown }
    }

    pub fn workers(&self) -> usize {
        self.senders.len() + 1
    }

    pub fn run(&self, work: &(dyn Fn() + Sync)) {
        if self.senders.is_empty() {
            work();
            return;
        }

        let job =
            Job(unsafe { std::mem::transmute::<*const (dyn Fn() + Sync), *const (dyn Fn() + Sync + 'static)>(work) });

        *self.countdown.remaining.lock().unwrap() = self.senders.len();
        for sender in &self.senders {
            sender.send(Job(job.0)).unwrap();
        }

        work();

        let mut remaining = self.countdown.remaining.lock().unwrap();
        while *remaining != 0 {
            remaining = self.countdown.finished.wait(remaining).unwrap();
        }
    }
}
