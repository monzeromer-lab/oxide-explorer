use gtk::prelude::*;

use super::file_entry::FileEntry;

pub fn name_sorter() -> gtk::CustomSorter {
    gtk::CustomSorter::new(move |a, b| {
        let a = a.downcast_ref::<FileEntry>().unwrap();
        let b = b.downcast_ref::<FileEntry>().unwrap();
        // Directories first, then alphabetical
        match (a.is_dir(), b.is_dir()) {
            (true, false) => gtk::Ordering::Smaller,
            (false, true) => gtk::Ordering::Larger,
            _ => a.name().to_lowercase().cmp(&b.name().to_lowercase()).into(),
        }
    })
}

pub fn size_sorter() -> gtk::CustomSorter {
    gtk::CustomSorter::new(move |a, b| {
        let a = a.downcast_ref::<FileEntry>().unwrap();
        let b = b.downcast_ref::<FileEntry>().unwrap();
        match (a.is_dir(), b.is_dir()) {
            (true, false) => gtk::Ordering::Smaller,
            (false, true) => gtk::Ordering::Larger,
            _ => a.size().cmp(&b.size()).into(),
        }
    })
}

pub fn date_sorter() -> gtk::CustomSorter {
    gtk::CustomSorter::new(move |a, b| {
        let a = a.downcast_ref::<FileEntry>().unwrap();
        let b = b.downcast_ref::<FileEntry>().unwrap();
        match (a.is_dir(), b.is_dir()) {
            (true, false) => gtk::Ordering::Smaller,
            (false, true) => gtk::Ordering::Larger,
            _ => a.modified().cmp(&b.modified()).into(),
        }
    })
}
