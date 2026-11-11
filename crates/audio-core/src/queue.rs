//! The bounded queue between capture callbacks and the audio pipeline.
//!
//! Capture callbacks run on real-time audio threads and must never block,
//! and memory use must stay bounded even if the consumer stalls. So when
//! the queue is full the *new* frame is dropped and counted, rather than
//! waiting or growing.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

use crate::frame::AudioFrame;

/// Creates a queue that holds at most `capacity` frames.
#[must_use]
pub fn frame_queue(capacity: usize) -> (FrameSender, FrameReceiver) {
    let (sender, receiver) = mpsc::sync_channel(capacity);
    let dropped = Arc::new(AtomicU64::new(0));
    (
        FrameSender {
            sender,
            dropped: Arc::clone(&dropped),
        },
        FrameReceiver { receiver, dropped },
    )
}

/// The producing end, used from capture callbacks. Cheap to clone, one per source.
#[derive(Debug, Clone)]
pub struct FrameSender {
    sender: SyncSender<AudioFrame>,
    dropped: Arc<AtomicU64>,
}

impl FrameSender {
    /// Queues a frame without blocking. If the queue is full, or the
    /// receiver is gone, the frame is discarded and counted as dropped.
    pub fn push(&self, frame: AudioFrame) {
        match self.sender.try_send(frame) {
            Ok(()) => {}
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// The consuming end, owned by the audio pipeline.
#[derive(Debug)]
pub struct FrameReceiver {
    receiver: Receiver<AudioFrame>,
    dropped: Arc<AtomicU64>,
}

impl FrameReceiver {
    /// Waits up to `timeout` for the next frame. Returns `None` if none
    /// arrived in time or every sender has been dropped.
    #[must_use]
    pub fn next(&self, timeout: Duration) -> Option<AudioFrame> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// Total number of frames dropped since the queue was created.
    #[must_use]
    pub fn dropped_frames(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::AudioSource;

    const NO_WAIT: Duration = Duration::ZERO;

    fn frame(marker: f32) -> AudioFrame {
        AudioFrame {
            source: AudioSource::Microphone,
            captured_at: Duration::ZERO,
            sample_rate: 48_000,
            samples: vec![marker],
        }
    }

    #[test]
    fn frames_arrive_in_order() {
        let (sender, receiver) = frame_queue(2);
        sender.push(frame(1.0));
        sender.push(frame(2.0));
        assert_eq!(receiver.next(NO_WAIT), Some(frame(1.0)));
        assert_eq!(receiver.next(NO_WAIT), Some(frame(2.0)));
        assert_eq!(receiver.next(NO_WAIT), None);
        assert_eq!(receiver.dropped_frames(), 0);
    }

    #[test]
    fn a_full_queue_drops_new_frames_and_counts_them() {
        let (sender, receiver) = frame_queue(1);
        sender.push(frame(1.0));
        sender.push(frame(2.0));
        sender.push(frame(3.0));
        assert_eq!(receiver.dropped_frames(), 2);
        assert_eq!(receiver.next(NO_WAIT), Some(frame(1.0)));
        assert_eq!(receiver.next(NO_WAIT), None);
    }

    #[test]
    fn pushing_after_the_receiver_is_gone_does_not_panic() {
        let (sender, receiver) = frame_queue(1);
        drop(receiver);
        sender.push(frame(1.0));
    }

    #[test]
    fn the_receiver_ends_when_every_sender_is_dropped() {
        let (sender, receiver) = frame_queue(1);
        let clone = sender.clone();
        drop(sender);
        drop(clone);
        assert_eq!(receiver.next(Duration::from_secs(5)), None);
    }
}
