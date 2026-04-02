use gio::prelude::*;
use std::path::Path;

pub struct DirectoryMonitor {
    _monitor: gio::FileMonitor,
}

impl DirectoryMonitor {
    pub fn new<F>(path: &Path, callback: F) -> Option<Self>
    where
        F: Fn() + 'static,
    {
        let file = gio::File::for_path(path);
        let monitor = file
            .monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
            .ok()?;

        monitor.connect_changed(move |_, _, _, _| {
            callback();
        });

        Some(Self { _monitor: monitor })
    }
}
