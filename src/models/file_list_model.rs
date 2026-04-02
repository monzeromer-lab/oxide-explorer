use gio::prelude::*;
use gio::subclass::prelude::*;
use std::cell::RefCell;

use super::file_entry::FileEntry;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct FileListModel {
        pub(super) items: RefCell<Vec<FileEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileListModel {
        const NAME: &'static str = "OxFileListModel";
        type Type = super::FileListModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for FileListModel {}

    impl ListModelImpl for FileListModel {
        fn item_type(&self) -> glib::Type {
            FileEntry::static_type()
        }

        fn n_items(&self) -> u32 {
            self.items.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.items
                .borrow()
                .get(position as usize)
                .map(|e| e.clone().upcast())
        }
    }
}

glib::wrapper! {
    pub struct FileListModel(ObjectSubclass<imp::FileListModel>)
        @implements gio::ListModel;
}

impl FileListModel {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn replace_all(&self, entries: Vec<FileEntry>) {
        let imp = self.imp();
        let old_len = imp.items.borrow().len() as u32;
        *imp.items.borrow_mut() = entries;
        let new_len = imp.items.borrow().len() as u32;
        self.items_changed(0, old_len, new_len);
    }

    pub fn clear(&self) {
        let imp = self.imp();
        let old_len = imp.items.borrow().len() as u32;
        imp.items.borrow_mut().clear();
        self.items_changed(0, old_len, 0);
    }
}
