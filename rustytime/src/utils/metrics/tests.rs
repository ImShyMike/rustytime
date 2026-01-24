use super::*;
use std::thread;

#[test]
fn metrics_tracker_record_request_increases_count() {
    let tracker = MetricsTracker::new();
    tracker.record_request();
    let recent = tracker.recent_requests.lock().unwrap();
    assert_eq!(recent.len(), 1);
}

#[test]
fn multiple_requests_increase_requests_per_second() {
    let tracker = MetricsTracker::new();
    for _ in 0..10 {
        tracker.record_request();
    }
    let metrics = tracker.get_metrics();
    assert_eq!(metrics.requests_per_second, 10.0 / 60.0);
}

#[test]
fn metrics_tracker_is_thread_safe() {
    let tracker = MetricsTracker::new();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let tracker_clone = tracker.clone();
            thread::spawn(move || {
                for _ in 0..25 {
                    tracker_clone.record_request();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let recent = tracker.recent_requests.lock().unwrap();
    assert_eq!(recent.len(), 100);
}
