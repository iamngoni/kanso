//! Drawing off the caller's thread, and keeping what was drawn.
//!
//! Two worker threads take jobs in the order they come; a job asked for
//! while it is being drawn waits for that drawing rather than starting
//! another. Finished drawings stay in a cache bounded by count and size,
//! the least recently used going first.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, mpsc};

use crate::{Job, Kind, Outcome, plugin, render};

/// The most drawings kept, and the most SVG they may hold between them.
const MAX_ENTRIES: usize = 768;
const MAX_BYTES: usize = 48 << 20;
const WORKERS: usize = 2;

type Waiter = Box<dyn FnOnce() + Send>;

struct Entry {
    outcome: Arc<Outcome>,
    used: u64,
    size: usize,
}

#[derive(Default)]
struct State {
    done: HashMap<Job, Entry>,
    pending: HashMap<Job, Vec<Waiter>>,
    tick: u64,
    bytes: usize,
}

impl State {
    fn insert(&mut self, job: Job, outcome: Arc<Outcome>) {
        let size = outcome.as_ref().as_ref().map_or(64, |svg| svg.markup.len()) + job.source.len();
        self.tick += 1;
        if let Some(old) = self.done.insert(
            job,
            Entry {
                outcome,
                used: self.tick,
                size,
            },
        ) {
            self.bytes -= old.size;
        }
        self.bytes += size;
        while self.done.len() > MAX_ENTRIES || (self.bytes > MAX_BYTES && self.done.len() > 1) {
            let Some(oldest) = self
                .done
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(job, _)| job.clone())
            else {
                break;
            };
            if let Some(entry) = self.done.remove(&oldest) {
                self.bytes -= entry.size;
            }
        }
    }
}

impl State {
    /// Drops every kept drawing of `kind`.
    fn forget(&mut self, kind: Kind) {
        let mut freed = 0;
        self.done.retain(|job, entry| {
            let keep = job.kind != kind;
            if !keep {
                freed += entry.size;
            }
            keep
        });
        self.bytes -= freed;
    }
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(Default::default);

/// Where jobs go: the workers, started on first use.
static QUEUE: LazyLock<Mutex<mpsc::Sender<Job>>> = LazyLock::new(|| {
    let (send, receive) = mpsc::channel::<Job>();
    let receive = Arc::new(Mutex::new(receive));
    for n in 0..WORKERS {
        let receive = receive.clone();
        std::thread::Builder::new()
            .name(format!("kanso-render-{n}"))
            .spawn(move || {
                loop {
                    let job = match receive.lock() {
                        Ok(queue) => match queue.recv() {
                            Ok(job) => job,
                            Err(_) => return,
                        },
                        Err(_) => return,
                    };
                    draw(job);
                }
            })
            .expect("starting a render thread");
    }
    Mutex::new(send)
});

/// Drops every kept plugin drawing, and makes one still being drawn for
/// the previous renderer not be kept. Called when the renderer changes.
pub fn forget_plugin_drawings() {
    forget_kind(Kind::Plugin);
}

/// Drops every kept drawing, of any kind: what was drawn from a locked
/// note's text or attachments must not outlive the lock keys.
pub fn forget_all() {
    plugin::next_generation();
    if let Ok(mut state) = STATE.lock() {
        state.done.clear();
        state.bytes = 0;
    }
}

/// Drops every kept drawing of `kind`. For plugin drawings, also moves the
/// generation on first, so a drawing in flight isn't kept afterwards.
pub(crate) fn forget_kind(kind: Kind) {
    if kind == Kind::Plugin {
        plugin::next_generation();
    }
    if let Ok(mut state) = STATE.lock() {
        state.forget(kind);
    }
}

/// Draws `job` and keeps the result. A plugin drawing made for a renderer
/// that has since changed is thrown away and made again.
fn draw(job: Job) {
    for attempt in 0..3 {
        let generation = plugin::generation();
        let outcome = std::panic::catch_unwind(|| render(&job))
            .unwrap_or_else(|_| Err(crate::Failure::Invalid("Drawing this failed.".into())));
        let last = attempt == 2;
        if finish(job.clone(), outcome, (!last).then_some(generation)) {
            return;
        }
    }
}

/// Keeps `outcome` and wakes whoever waited for it. Given the generation
/// a plugin drawing was made in, keeps nothing and returns false when that
/// is no longer the current one.
fn finish(job: Job, outcome: Outcome, generation: Option<u64>) -> bool {
    let waiters = {
        let Ok(mut state) = STATE.lock() else {
            return true;
        };
        if job.kind == Kind::Plugin && generation.is_some_and(|g| g != plugin::generation()) {
            return false;
        }
        state.insert(job.clone(), Arc::new(outcome));
        state.pending.remove(&job).unwrap_or_default()
    };
    for waiter in waiters {
        waiter();
    }
    true
}

/// The drawing for `job`, if it has been made and is still kept.
pub fn cached(job: &Job) -> Option<Arc<Outcome>> {
    let mut state = STATE.lock().ok()?;
    state.tick += 1;
    let tick = state.tick;
    let entry = state.done.get_mut(job)?;
    entry.used = tick;
    Some(entry.outcome.clone())
}

/// Draws `job` on a worker thread unless it is drawn or being drawn, and
/// calls `done` (on some other thread) once [`cached`] has it.
pub fn request(job: Job, done: impl FnOnce() + Send + 'static) {
    let Ok(mut state) = STATE.lock() else { return };
    if state.done.contains_key(&job) {
        drop(state);
        done();
        return;
    }
    if let Some(waiters) = state.pending.get_mut(&job) {
        waiters.push(Box::new(done));
        return;
    }
    state.pending.insert(job.clone(), vec![Box::new(done)]);
    drop(state);
    let sent = QUEUE.lock().map(|queue| queue.send(job.clone()).is_ok());
    if !matches!(sent, Ok(true)) {
        // No workers: draw here rather than never.
        draw(job);
    }
}

/// The drawing for `job`, made now if need be. Blocks; for exports.
pub fn render_blocking(job: &Job) -> Arc<Outcome> {
    if let Some(outcome) = cached(job) {
        return outcome;
    }
    let (send, receive) = mpsc::channel();
    request(job.clone(), move || {
        send.send(()).ok();
    });
    receive.recv().ok();
    cached(job).unwrap_or_else(|| Arc::new(render(job)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Kind, Palette};

    #[test]
    fn requests_are_drawn_once_and_kept() {
        let job = Job::new(
            Kind::Mermaid,
            "graph TD\n  cache-a --> cache-b\n",
            Palette::LIGHT,
        );
        let (send, receive) = mpsc::channel();
        for _ in 0..3 {
            let send = send.clone();
            request(job.clone(), move || {
                send.send(()).ok();
            });
        }
        for _ in 0..3 {
            receive.recv().unwrap();
        }
        let first = cached(&job).unwrap();
        assert!(first.is_ok());
        let again = render_blocking(&job);
        assert!(Arc::ptr_eq(&first, &again));
    }

    #[test]
    fn the_cache_stays_bounded() {
        let mut state = State::default();
        for n in 0..(MAX_ENTRIES + 50) {
            let job = Job::new(Kind::InlineMath, format!("x_{n}"), Palette::LIGHT);
            state.insert(job, Arc::new(Err(crate::Failure::Invalid(String::new()))));
        }
        assert_eq!(state.done.len(), MAX_ENTRIES);
        // The oldest went first.
        let first = Job::new(Kind::InlineMath, "x_0", Palette::LIGHT);
        let last = Job::new(
            Kind::InlineMath,
            format!("x_{}", MAX_ENTRIES + 49),
            Palette::LIGHT,
        );
        assert!(!state.done.contains_key(&first));
        assert!(state.done.contains_key(&last));
        let total: usize = state.done.values().map(|e| e.size).sum();
        assert_eq!(total, state.bytes);
    }
}
