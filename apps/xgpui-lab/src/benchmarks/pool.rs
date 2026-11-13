//! Bounded independent-case execution; results retain their planned order.
use super::*;
use std::sync::atomic::AtomicUsize;
pub(crate) fn run<T: Sync, R: Send>(
    tasks: &[T],
    workers: usize,
    cancel: &AtomicBool,
    work: impl Fn(usize, usize, &T) -> R + Sync,
) -> Result<Vec<R>> {
    ensure!(matches!(workers, 1 | 2), "expected one or two SVG workers");
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..workers.min(tasks.len()) {
            let next = &next;
            let work = &work;
            handles.push(scope.spawn(move || {
                let mut results = Vec::new();
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(task) = tasks.get(index) else {
                        break;
                    };
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    results.push((index, work(worker, index, task)));
                }
                results
            }));
        }
        let mut results = Vec::new();
        for handle in handles {
            results.extend(handle.join().map_err(|_| anyhow::anyhow!("SVG case worker panicked"))?);
        }
        results.sort_by_key(|(index, _)| *index);
        Ok(results.into_iter().map(|(_, report)| report).collect())
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn sequential_and_parallel_obey_worker_limits_and_keep_case_order() {
        for workers in [1, 2] {
            let active = AtomicUsize::new(0);
            let peak = AtomicUsize::new(0);
            let barrier = std::sync::Barrier::new(workers);
            let values = run(&[0, 1, 2, 3, 4, 5], workers, &AtomicBool::new(false), |_, index, value| {
                let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(count, Ordering::SeqCst);
                if index < workers {
                    barrier.wait();
                }
                std::thread::sleep(Duration::from_millis(2));
                active.fetch_sub(1, Ordering::SeqCst);
                *value
            })
            .unwrap();
            assert_eq!(values, vec![0, 1, 2, 3, 4, 5]);
            assert_eq!(peak.load(Ordering::SeqCst), workers);
        }
    }
    #[::core::prelude::v1::test]
    fn cancellation_stops_dequeuing_pending_cases() {
        let cancel = AtomicBool::new(false);
        let values = run(&[0, 1, 2, 3, 4, 5], 2, &cancel, |_, _, value| {
            cancel.store(true, Ordering::Relaxed);
            *value
        })
        .unwrap();
        assert!(!values.is_empty());
        assert!(values.len() <= 2);
        assert!(run(&[1], 3, &cancel, |_, _, v| *v).is_err());
    }
}
