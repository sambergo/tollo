//! Coordinates complete command workflows, including work outside the database lock.
use std::sync::{Arc, Condvar, Mutex, OnceLock};

#[derive(Default)]
struct Counts {
    readers: usize,
    writers_waiting: usize,
    writing: bool,
}

#[derive(Default)]
pub(crate) struct OperationGate {
    counts: Mutex<Counts>,
    changed: Condvar,
}

pub(crate) struct Guard {
    gate: Arc<OperationGate>,
    exclusive: bool,
}

impl OperationGate {
    fn acquire(self: &Arc<Self>, exclusive: bool) -> Result<Guard, String> {
        let mut counts = self.counts.lock().map_err(|e| e.to_string())?;
        if exclusive {
            counts.writers_waiting += 1;
            while counts.writing || counts.readers > 0 {
                counts = self.changed.wait(counts).map_err(|e| e.to_string())?;
            }
            counts.writers_waiting -= 1;
            counts.writing = true;
        } else {
            while counts.writing || counts.writers_waiting > 0 {
                counts = self.changed.wait(counts).map_err(|e| e.to_string())?;
            }
            counts.readers += 1;
        }
        Ok(Guard {
            gate: self.clone(),
            exclusive,
        })
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Ok(mut counts) = self.gate.counts.lock() {
            if self.exclusive {
                counts.writing = false;
            } else {
                counts.readers -= 1;
            }
            self.gate.changed.notify_all();
        }
    }
}

fn gate() -> &'static Arc<OperationGate> {
    static GATE: OnceLock<Arc<OperationGate>> = OnceLock::new();
    GATE.get_or_init(|| Arc::new(OperationGate::default()))
}

pub(crate) fn read() -> Result<Guard, String> {
    gate().acquire(false)
}
pub(crate) fn write() -> Result<Guard, String> {
    gate().acquire(true)
}
pub(crate) async fn read_async() -> Result<Guard, String> {
    tauri::async_runtime::spawn_blocking(read)
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    #[test]
    fn restore_waits_for_active_operations_and_blocks_new_ones() {
        let gate = Arc::new(OperationGate::default());
        let active = gate.acquire(false).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_gate = gate.clone();
        let writer = std::thread::spawn(move || {
            let _guard = writer_gate.acquire(true).unwrap();
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        assert!(started_rx.recv_timeout(Duration::from_millis(30)).is_err());
        drop(active);
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (reader_tx, reader_rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            let _guard = gate.acquire(false).unwrap();
            reader_tx.send(()).unwrap();
        });
        assert!(reader_rx.recv_timeout(Duration::from_millis(30)).is_err());
        release_tx.send(()).unwrap();
        reader_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        writer.join().unwrap();
        reader.join().unwrap();
    }
}
