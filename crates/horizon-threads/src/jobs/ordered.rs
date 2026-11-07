//! Parallel background work consumed in submission order, without waiting in a frame.
use super::{Class, Pool, Rank, Registration, Source, lock};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender, TryRecvError},
};

type Job<T> = (Box<dyn FnOnce() -> T + Send>, Sender<T>);

struct Queue<T> {
    jobs: Mutex<VecDeque<Job<T>>>,
    outstanding: AtomicUsize,
    window: usize,
    class: Class,
}

impl<T: Send + 'static> Source for Queue<T> {
    fn next(&self) -> Option<Rank> {
        let jobs = lock(&self.jobs);
        (self.outstanding.load(Ordering::Acquire) < self.window && !jobs.is_empty())
            .then_some(Rank::new(self.class, 0))
    }

    fn run(&self, _: usize) -> bool {
        let job = {
            let mut jobs = lock(&self.jobs);
            if self.outstanding.load(Ordering::Acquire) >= self.window {
                return false;
            }
            let Some(job) = jobs.pop_front() else {
                return false;
            };
            self.outstanding.fetch_add(1, Ordering::AcqRel);
            job
        };
        let (work, done) = job;
        let _ = done.send(work());
        true
    }
}

/// A connection's ordered decoder, sharing the process's workers. `window`
/// bounds running plus completed-but-unconsumed results: a stalled consumer
/// never lets the workers expand its entire packet backlog into decoded data.
pub struct Ordered<T: Send + 'static> {
    queue: Arc<Queue<T>>,
    results: RefCell<VecDeque<Receiver<T>>>,
    registration: Registration,
}

impl<T: Send + 'static> Ordered<T> {
    pub fn new(pool: &Pool, class: Class, window: usize) -> Self {
        let queue = Arc::new(Queue {
            jobs: Mutex::new(VecDeque::new()),
            outstanding: AtomicUsize::new(0),
            window: window.max(1),
            class,
        });
        let registration = pool.register(queue.clone());
        Self {
            queue,
            results: RefCell::new(VecDeque::new()),
            registration,
        }
    }

    pub fn submit(&self, job: impl FnOnce() -> T + Send + 'static) {
        let (done, result) = mpsc::channel();
        self.results.borrow_mut().push_back(result);
        lock(&self.queue.jobs).push_back((Box::new(job), done));
        self.registration.notify();
    }

    /// Only the oldest result is eligible, even if later jobs finished first.
    /// A failed job releases its slot too, allowing the connection to recover.
    pub fn try_next(&self) -> Result<T, TryRecvError> {
        let mut results = self.results.borrow_mut();
        let result = results.front().ok_or(TryRecvError::Empty)?.try_recv();
        if !matches!(&result, Err(TryRecvError::Empty)) {
            results.pop_front();
            self.queue.outstanding.fetch_sub(1, Ordering::AcqRel);
            self.registration.notify();
        }
        result
    }
}

impl<T: Send + 'static> Drop for Ordered<T> {
    fn drop(&mut self) {
        // Queued work belongs to this connection, not the next server joined.
        lock(&self.queue.jobs).clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn receive<T: Send>(jobs: &Ordered<T>) -> T {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            match jobs.try_next() {
                Ok(value) => return value,
                Err(TryRecvError::Empty) if Instant::now() < end => std::thread::yield_now(),
                _ => panic!("ordered job did not finish"),
            }
        }
    }

    #[test]
    fn jobs_run_in_parallel_but_results_keep_submission_order() {
        let pool = Pool::new(2, "ordered-test");
        let jobs = Ordered::new(&pool, Class::Urgent, 2);
        let (release, held) = mpsc::channel();
        let (finished, second) = mpsc::channel();
        jobs.submit(move || {
            held.recv().unwrap();
            1
        });
        jobs.submit(move || {
            finished.send(()).unwrap();
            2
        });
        // A single worker could not finish the second job before the first.
        second.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(jobs.try_next(), Err(TryRecvError::Empty));
        release.send(()).unwrap();
        assert_eq!((receive(&jobs), receive(&jobs)), (1, 2));
    }

    #[test]
    fn completed_results_count_towards_the_window() {
        let pool = Pool::new(3, "ordered-window");
        let jobs = Ordered::new(&pool, Class::Urgent, 2);
        let (finished, done) = mpsc::channel();
        for n in 0..4 {
            let finished = finished.clone();
            jobs.submit(move || {
                finished.send(n).unwrap();
                n
            });
        }
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(done.try_recv(), Err(TryRecvError::Empty));
        assert_eq!(receive(&jobs), 0);
        assert_eq!(done.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
        assert_eq!((receive(&jobs), receive(&jobs), receive(&jobs)), (1, 2, 3));
    }

    #[test]
    fn a_panicking_job_does_not_block_later_results() {
        let pool = Pool::new(2, "ordered-panic");
        let jobs = Ordered::new(&pool, Class::Urgent, 2);
        jobs.submit(|| panic!("failed decode"));
        jobs.submit(|| 42);
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            match jobs.try_next() {
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) if Instant::now() < end => std::thread::yield_now(),
                _ => panic!("missing failed-job result"),
            }
        }
        assert_eq!(receive(&jobs), 42);
    }
}
