use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::FileEntry)]
    pub struct FileEntry {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        path: RefCell<String>,
        #[property(get, set)]
        size: Cell<u64>,
        #[property(get, set)]
        modified: Cell<i64>,
        #[property(get, set)]
        is_dir: Cell<bool>,
        #[property(get, set)]
        icon_name: RefCell<String>,
        #[property(get, set)]
        content_type: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileEntry {
        const NAME: &'static str = "OxFileEntry";
        type Type = super::FileEntry;
    }

    #[glib::derived_properties]
    impl ObjectImpl for FileEntry {}
}

glib::wrapper! {
    pub struct FileEntry(ObjectSubclass<imp::FileEntry>);
}

impl FileEntry {
    pub fn new(
        name: &str,
        path: &str,
        size: u64,
        modified: i64,
        is_dir: bool,
        icon_name: &str,
        content_type: &str,
    ) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("path", path)
            .property("size", size)
            .property("modified", modified)
            .property("is-dir", is_dir)
            .property("icon-name", icon_name)
            .property("content-type", content_type)
            .build()
    }
}
