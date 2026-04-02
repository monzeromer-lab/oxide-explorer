use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;
use crate::models::file_list_model::FileListModel;
use crate::operations::monitor::DirectoryMonitor;
use crate::state::navigation::NavigationState;
use crate::widgets::content_view::ContentView;
use crate::widgets::filter_bar::FilterBar;
use crate::widgets::status_bar::StatusBar;

/// Per-tab state. Each tab has its own navigation, model, views, filter, etc.
pub struct TabState {
    pub nav: Rc<RefCell<NavigationState>>,
    pub file_model: FileListModel,
    pub filter: gtk::CustomFilter,
    pub filter_model: gtk::FilterListModel,
    pub selection_model: gtk::MultiSelection,
    pub content: Rc<ContentView>,
    pub status_bar: Rc<StatusBar>,
    pub filter_bar: Rc<FilterBar>,
    pub monitor: Rc<RefCell<Option<DirectoryMonitor>>>,
    /// The container widget that holds filter_bar + content + status_bar
    pub container: gtk::Box,
    #[allow(dead_code)]
    pub filter_text: Rc<RefCell<String>>,
}

impl TabState {
    pub fn new(
        start_path: PathBuf,
        show_hidden: Rc<Cell<bool>>,
        icon_size: Rc<Cell<i32>>,
    ) -> Self {
        let nav = Rc::new(RefCell::new(NavigationState::new(start_path)));
        let file_model = FileListModel::new();
        let monitor = Rc::new(RefCell::new(None));

        // Filter
        let filter = gtk::CustomFilter::new(|_| true);
        let filter_model =
            gtk::FilterListModel::new(Some(file_model.clone()), Some(filter.clone()));
        let selection_model = gtk::MultiSelection::new(Some(filter_model.clone()));

        // Filter text
        let filter_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        // Set up filter function
        let sh = show_hidden.clone();
        let ft = filter_text.clone();
        filter.set_filter_func(move |item| {
            let entry = item.downcast_ref::<FileEntry>().unwrap();
            let name = entry.name();
            if !sh.get() && name.starts_with('.') {
                return false;
            }
            let ft = ft.borrow();
            if ft.is_empty() {
                return true;
            }
            name.to_lowercase().contains(ft.as_str())
        });

        // Widgets
        let content = Rc::new(ContentView::new(&selection_model, icon_size));
        let status_bar = Rc::new(StatusBar::new());
        let filter_bar = Rc::new(FilterBar::new());

        // Wire filter bar to filter
        let ft2 = filter_text.clone();
        let filter_ref = filter.clone();
        filter_bar.entry.connect_search_changed(move |entry| {
            *ft2.borrow_mut() = entry.text().to_string().to_lowercase();
            filter_ref.changed(gtk::FilterChange::Different);
        });

        // Wire selection tracking to status bar
        let status_for_sel = status_bar.clone();
        selection_model.connect_selection_changed(move |sel, _, _| {
            let mut count = 0u32;
            for i in 0..sel.n_items() {
                if sel.is_selected(i) {
                    count += 1;
                }
            }
            status_for_sel.set_selection_info(count);
        });

        // Container — must fill the entire tab page
        content.outer_stack.set_vexpand(true);
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.append(&filter_bar.widget);
        container.append(&content.outer_stack);
        container.append(&status_bar.widget);

        Self {
            nav,
            file_model,
            filter,
            filter_model,
            selection_model,
            content,
            status_bar,
            filter_bar,
            monitor,
            container,
            filter_text,
        }
    }

    pub fn current_path(&self) -> PathBuf {
        self.nav.borrow().current.clone()
    }

}

use gtk::prelude::*;
