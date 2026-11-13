//! Owns the disk index: loads the cached one at launch, runs scans on a
//! background thread, and derives the semantic overview.

use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tessera_core::disk::changes::{self, DiskChanges};
use tessera_core::disk::{self, DiskIndex, Overview, ScanConfig, ScanPriority, ScanProgress};
use tessera_core::metrics::Volume;
use tessera_core::paths;

pub struct ActiveScan {
    pub progress: Arc<ScanProgress>,
    pub started: Instant,
}

#[derive(Clone)]
pub enum ChangesState {
    Loading,
    Unavailable(String),
    Ready(Arc<DiskChanges>),
}

#[derive(Clone, Copy)]
struct SavedIndex {
    slot: usize,
    sequence: u64,
}

#[derive(Default)]
struct Inner {
    index: Option<Arc<DiskIndex>>,
    overview: Option<Arc<Overview>>,
    scan: Option<Arc<ActiveScan>>,
    error: Option<String>,
    generation: u64,
    volume: Volume,
    // Advances after each completed walk, including an unsuccessful cache write.
    latest_sequence: u64,
    saved: Option<SavedIndex>,
    previous: Option<SavedIndex>,
    changes: Option<ChangesState>,
}

/// A cheap, cloneable view of the current disk state for the UI.
#[derive(Clone, Default)]
pub struct DiskView {
    pub index: Option<Arc<DiskIndex>>,
    pub overview: Option<Arc<Overview>>,
    pub scan: Option<Arc<ActiveScan>>,
    pub error: Option<String>,
    pub generation: u64,
}

pub struct DiskService {
    inner: Mutex<Inner>,
    files: [PathBuf; 2],
}

impl DiskService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner::default()),
            files: [paths::index_file(), paths::alternate_index_file()],
        })
    }

    pub fn view(&self) -> DiskView {
        let i = self.inner.lock().unwrap();
        DiskView {
            index: i.index.clone(),
            overview: i.overview.clone(),
            scan: i.scan.clone(),
            error: i.error.clone(),
            generation: i.generation,
        }
    }

    /// Load the index saved by the previous run, if any.
    pub fn load_cached(&self) -> bool {
        let sequences = self
            .files
            .each_ref()
            .map(|p| DiskIndex::stored_sequence(p).ok());
        let newest = usize::from(sequences[1] > sequences[0]);
        for slot in [newest, 1 - newest] {
            let Some(sequence) = sequences[slot] else {
                continue;
            };
            let Ok((actual, index)) = DiskIndex::load_with_sequence(&self.files[slot]) else {
                continue;
            };
            if sequence != actual {
                continue;
            }
            let mut i = self.inner.lock().unwrap();
            i.overview = Some(Arc::new(build_overview(&index, &i.volume)));
            i.index = Some(Arc::new(index));
            i.saved = Some(SavedIndex { slot, sequence });
            i.latest_sequence = sequence;
            i.previous = sequences[1 - slot]
                .filter(|prev| prev.checked_add(1) == Some(sequence))
                .map(|sequence| SavedIndex {
                    slot: 1 - slot,
                    sequence,
                });
            i.changes = None;
            i.generation += 1;
            return true;
        }
        false
    }

    #[cfg(test)]
    pub fn set_index(&self, index: DiskIndex) {
        let mut i = self.inner.lock().unwrap();
        let volume = i.volume.clone();
        i.overview = Some(Arc::new(build_overview(&index, &volume)));
        i.index = Some(Arc::new(index));
        i.generation += 1;
    }

    /// Decode the old slot only when the user opens Changes. Cache only the
    /// resulting short list and keep its node IDs tied to this exact scan.
    pub fn changes(
        self: &Arc<Self>,
        current: &Arc<DiskIndex>,
        on_done: impl FnOnce() + Send + 'static,
    ) -> ChangesState {
        let (previous, overview) = {
            let mut i = self.inner.lock().unwrap();
            if !i
                .index
                .as_ref()
                .is_some_and(|idx| Arc::ptr_eq(idx, current))
            {
                return ChangesState::Loading;
            }
            if let Some(state) = &i.changes {
                return state.clone();
            }
            let Some(previous) = i.previous else {
                return ChangesState::Unavailable(
                    "Growth appears after two scans with matching coverage. Use Rescan to take the next one.".into(),
                );
            };
            let overview = i
                .overview
                .clone()
                .expect("an installed scan has an overview");
            i.changes = Some(ChangesState::Loading);
            (previous, overview)
        };
        let this = self.clone();
        let worker_index = current.clone();
        let spawn = std::thread::Builder::new()
            .name("tessera-disk-changes".into())
            .spawn(move || {
                crate::macos::set_thread_qos_utility();
                let comparison = this.compare_saved(previous, &worker_index, &overview);
                let mut i = this.inner.lock().unwrap();
                if i.index
                    .as_ref()
                    .is_some_and(|idx| Arc::ptr_eq(idx, &worker_index))
                {
                    i.changes = Some(comparison);
                }
                drop(i);
                on_done();
            });
        if let Err(e) = spawn {
            let state = ChangesState::Unavailable(format!("Could not open disk changes: {e}"));
            let mut i = self.inner.lock().unwrap();
            if i.index
                .as_ref()
                .is_some_and(|idx| Arc::ptr_eq(idx, current))
            {
                i.changes = Some(state.clone());
            }
            return state;
        }
        ChangesState::Loading
    }

    fn compare_saved(
        &self,
        previous: SavedIndex,
        current: &DiskIndex,
        overview: &Overview,
    ) -> ChangesState {
        let old = match DiskIndex::load_with_sequence(&self.files[previous.slot]) {
            Ok((seq, index)) if seq == previous.sequence => index,
            _ => {
                return ChangesState::Unavailable(
                    "The previous scan could not be read. Rescan to compare again.".into(),
                );
            }
        };
        if !current.has_same_coverage(&old) {
            return ChangesState::Unavailable("The previous scan has different or unknown coverage. Rescan to compare the same locations.".into());
        }
        ChangesState::Ready(Arc::new(changes::compare(&old, current, overview)))
    }

    /// Keep the volume numbers fresh so the "hidden space" block is right.
    pub fn set_volume(&self, volume: &Volume) {
        let mut i = self.inner.lock().unwrap();
        let changed = i.volume.total != volume.total
            || i.volume.available.abs_diff(volume.available) > 512 << 20;
        i.volume = volume.clone();
        if changed && let Some(index) = i.index.clone() {
            i.overview = Some(Arc::new(build_overview(&index, volume)));
            i.generation += 1;
        }
    }

    pub fn index_age(&self) -> Option<Duration> {
        self.inner
            .lock()
            .unwrap()
            .index
            .as_ref()
            .map(|i| disk::scan::age(i.scanned_at))
    }

    fn publish_scan(&self, index: DiskIndex) -> io::Result<()> {
        let next = {
            let i = self.inner.lock().unwrap();
            SavedIndex {
                // Replace the older file, even if the latest walk failed to save.
                slot: i.saved.map_or(0, |saved| 1 - saved.slot),
                sequence: i
                    .latest_sequence
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("invalid disk index sequence"))?,
            }
        };
        let written = index.save_with_sequence(&self.files[next.slot], next.sequence);
        let mut i = self.inner.lock().unwrap();
        i.previous = i.saved.filter(|saved| saved.sequence == i.latest_sequence);
        i.latest_sequence = next.sequence;
        if written.is_ok() {
            i.saved = Some(next);
        }
        i.overview = Some(Arc::new(build_overview(&index, &i.volume)));
        i.index = Some(Arc::new(index));
        i.changes = None;
        i.generation += 1;
        written.map_err(|e| io::Error::other(format!("Could not save disk scan: {e}")))
    }

    /// Start a scan unless one is already running. `on_done` runs on the
    /// scanner thread once the new index is in place.
    pub fn start_scan(
        self: &Arc<Self>,
        priority: ScanPriority,
        exclude: Vec<PathBuf>,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        let progress = Arc::new(ScanProgress::default());
        {
            let mut i = self.inner.lock().unwrap();
            if i.scan.is_some() {
                return false;
            }
            i.scan = Some(Arc::new(ActiveScan {
                progress: progress.clone(),
                started: Instant::now(),
            }));
            i.error = None;
        }
        let this = self.clone();
        std::thread::Builder::new()
            .name("tessera-scan".into())
            .spawn(move || {
                let mut cfg = scan_config();
                cfg.priority = priority;
                cfg.exclude.extend(exclude);
                let result = disk::scan(&cfg, &progress).and_then(|index| this.publish_scan(index));
                {
                    let mut i = this.inner.lock().unwrap();
                    i.scan = None;
                    i.error = result.err().map(|e| e.to_string());
                }
                on_done();
            })
            .expect("spawn scan thread");
        true
    }
}

fn scan_config() -> ScanConfig {
    // TESSERA_SCAN_ROOT lets development builds scan a small folder instead
    // of the whole volume.
    let mut cfg = match std::env::var_os("TESSERA_SCAN_ROOT") {
        Some(root) => ScanConfig::for_path(root),
        None => ScanConfig::data_volume(),
    };
    if let Some(ex) = std::env::var_os("TESSERA_EXCLUDE") {
        cfg.exclude.extend(std::env::split_paths(&ex));
    }
    cfg
}

fn build_overview(index: &DiskIndex, volume: &Volume) -> Overview {
    let used = if volume.total > 0 {
        volume.used()
    } else {
        index.total()
    };
    disk::overview::build(index, used, &paths::home_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_cache_loads_previous_only_on_demand_and_recovers_latest_map() {
        let temp =
            std::env::temp_dir().join(format!("tessera-growth-cache-{}", std::process::id()));
        let root = temp.join("root");
        std::fs::create_dir_all(&root).unwrap();
        let service = Arc::new(DiskService {
            inner: Mutex::default(),
            files: [temp.join("one.bin"), temp.join("two.bin")],
        });
        let cfg = ScanConfig {
            threads: 1,
            ..ScanConfig::for_path(&root)
        };
        let old = disk::scan(&cfg, &ScanProgress::default()).unwrap();
        old.save_with_sequence(&service.files[1], 2).unwrap();
        let current = disk::scan(&cfg, &ScanProgress::default()).unwrap();
        current.save_with_sequence(&service.files[0], 3).unwrap();
        assert!(service.load_cached());
        let index = service.view().index.unwrap();
        assert_eq!(index.scanned_at, current.scanned_at);
        assert!(service.inner.lock().unwrap().changes.is_none());

        let (send, done) = std::sync::mpsc::channel();
        assert!(matches!(
            service.changes(&index, move || send.send(()).unwrap()),
            ChangesState::Loading
        ));
        done.recv_timeout(Duration::from_secs(5)).unwrap();
        let ChangesState::Ready(growth) = service.changes(&index, || unreachable!()) else {
            panic!("expected comparison of consecutive scans")
        };
        assert_eq!(growth.previous_at, old.scanned_at);
        assert_eq!(growth.current_at, current.scanned_at);

        // Even with an intact header, a truncated newer slot cannot replace
        // the last good map or be used as its previous scan.
        std::fs::OpenOptions::new()
            .write(true)
            .open(&service.files[0])
            .unwrap()
            .set_len(20)
            .unwrap();
        assert!(service.load_cached());
        let recovered = service.view().index.unwrap();
        assert_eq!(recovered.scanned_at, old.scanned_at);
        assert!(matches!(
            service.changes(&recovered, || unreachable!()),
            ChangesState::Unavailable(_)
        ));

        // Fail an atomic replacement after a successful walk. The live scan
        // can compare to the last saved one, but its successor needs a fresh
        // baseline because that intermediate scan is unavailable on disk.
        std::fs::remove_file(&service.files[0]).unwrap();
        std::fs::create_dir(&service.files[0]).unwrap();
        assert!(service.publish_scan(current.clone()).is_err());
        assert!(service.inner.lock().unwrap().previous.is_some());
        std::fs::remove_dir(&service.files[0]).unwrap();
        service.publish_scan(current.clone()).unwrap();
        assert!(service.inner.lock().unwrap().previous.is_none());
        assert!(service.load_cached());
        assert!(service.inner.lock().unwrap().previous.is_none());

        service.publish_scan(current).unwrap();
        assert!(service.load_cached());
        assert!(service.inner.lock().unwrap().previous.is_some());
        std::fs::remove_dir_all(temp).unwrap();
    }
}
