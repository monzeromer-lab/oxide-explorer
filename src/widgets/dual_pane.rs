use adw::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;
use crate::models::file_list_model::FileListModel;
use crate::operations::monitor::DirectoryMonitor;
use crate::state::navigation::NavigationState;
use crate::widgets::content_view::ContentView;
use crate::widgets::status_bar::StatusBar;

/// One side of the dual-pane view
pub struct PaneState {
    pub nav: Rc<RefCell<NavigationState>>,
    pub file_model: FileListModel,
    pub filter: gtk::CustomFilter,
    pub filter_model: gtk::FilterListModel,
    pub selection_model: gtk::MultiSelection,
    pub content: Rc<ContentView>,
    pub status_bar: Rc<StatusBar>,
    pub monitor: Rc<RefCell<Option<DirectoryMonitor>>>,
    pub container: gtk::Box,
    pub path_label: gtk::Label,
}

impl PaneState {
    pub fn new(path: PathBuf, show_hidden: Rc<Cell<bool>>, icon_size: Rc<Cell<i32>>) -> Self {
        let nav = Rc::new(RefCell::new(NavigationState::new(path.clone())));
        let file_model = FileListModel::new();
        let monitor = Rc::new(RefCell::new(None));

        let filter = gtk::CustomFilter::new(|_| true);
        let filter_model = gtk::FilterListModel::new(Some(file_model.clone()), Some(filter.clone()));
        let selection_model = gtk::MultiSelection::new(Some(filter_model.clone()));

        let sh = show_hidden.clone();
        filter.set_filter_func(move |item| {
            let entry = item.downcast_ref::<FileEntry>().unwrap();
            if !sh.get() && entry.name().starts_with('.') { return false; }
            true
        });

        let content = Rc::new(ContentView::new(&selection_model, icon_size));
        let status_bar = Rc::new(StatusBar::new());

        let path_label = gtk::Label::new(Some(&path.to_string_lossy()));
        path_label.set_halign(gtk::Align::Start);
        path_label.set_ellipsize(gtk::pango::EllipsizeMode::Start);
        path_label.add_css_class("dim-label");
        path_label.set_margin_start(8);
        path_label.set_margin_top(4);
        path_label.set_margin_bottom(2);

        content.outer_stack.set_vexpand(true);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&path_label);
        container.append(&content.outer_stack);
        container.append(&status_bar.widget);
        container.set_hexpand(true);
        container.set_vexpand(true);

        Self { nav, file_model, filter, filter_model, selection_model, content, status_bar, monitor, container, path_label }
    }

    pub fn current_path(&self) -> PathBuf {
        self.nav.borrow().current.clone()
    }

    pub fn load_directory(&self, path: PathBuf) {
        self.nav.borrow_mut().navigate_to(path.clone());
        self.content.show_loading();
        self.path_label.set_text(&path.to_string_lossy());

        let model = self.file_model.clone();
        let status = self.status_bar.clone();
        let content = self.content.clone();
        let filter = self.filter.clone();
        let filter_model = self.filter_model.clone();
        let monitor = self.monitor.clone();

        crate::window::load_path_async_pub(path, model, status, monitor, content, filter, filter_model);
    }

    pub fn get_selected_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let sel = &self.selection_model;
        for i in 0..sel.n_items() {
            if sel.is_selected(i) {
                if let Some(item) = sel.item(i) {
                    if let Some(entry) = item.downcast_ref::<FileEntry>() {
                        paths.push(PathBuf::from(entry.path()));
                    }
                }
            }
        }
        paths
    }
}

pub struct DualPane {
    pub paned: gtk::Paned,
    pub left: Rc<PaneState>,
    pub right: Rc<PaneState>,
    pub active_side: Rc<Cell<Side>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl DualPane {
    pub fn new(show_hidden: Rc<Cell<bool>>, icon_size: Rc<Cell<i32>>) -> Self {
        let home = glib::home_dir();
        let left = Rc::new(PaneState::new(home.clone(), show_hidden.clone(), icon_size.clone()));
        let right = Rc::new(PaneState::new(home, show_hidden.clone(), icon_size));

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_start_child(Some(&left.container));
        paned.set_end_child(Some(&right.container));
        paned.set_position(500);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        let active_side = Rc::new(Cell::new(Side::Left));

        // Click to focus a pane — update visual indicator
        let active = active_side.clone();
        let left_c = left.container.clone();
        let right_c = right.container.clone();
        let click_left = gtk::GestureClick::new();
        click_left.connect_pressed(move |_, _, _, _| {
            active.set(Side::Left);
            left_c.add_css_class("pane-active");
            right_c.remove_css_class("pane-active");
        });
        left.container.add_controller(click_left);

        let active = active_side.clone();
        let left_c = left.container.clone();
        let right_c = right.container.clone();
        let click_right = gtk::GestureClick::new();
        click_right.connect_pressed(move |_, _, _, _| {
            active.set(Side::Right);
            right_c.add_css_class("pane-active");
            left_c.remove_css_class("pane-active");
        });
        right.container.add_controller(click_right);

        // Default: left is active
        left.container.add_css_class("pane-active");

        // Wire double-click navigation in each pane
        Self::wire_pane_activate(&left, show_hidden.clone());
        Self::wire_pane_activate(&right, show_hidden.clone());

        Self { paned, left, right, active_side }
    }

    fn wire_pane_activate(pane: &Rc<PaneState>, _show_hidden: Rc<Cell<bool>>) {
        // Icon view double-click
        let pane_ref = pane.clone();
        pane.content.icon_view.grid_view.connect_activate(move |_, pos| {
            if let Some(item) = pane_ref.selection_model.item(pos) {
                if let Some(entry) = item.downcast_ref::<FileEntry>() {
                    let path = PathBuf::from(entry.path());
                    if entry.is_dir() {
                        pane_ref.load_directory(path);
                    } else {
                        crate::window::open_file(&path);
                    }
                }
            }
        });

        // Details view double-click
        let pane_ref = pane.clone();
        pane.content.details_view.column_view.connect_activate(move |_, pos| {
            if let Some(item) = pane_ref.selection_model.item(pos) {
                if let Some(entry) = item.downcast_ref::<FileEntry>() {
                    let path = PathBuf::from(entry.path());
                    if entry.is_dir() {
                        pane_ref.load_directory(path);
                    } else {
                        crate::window::open_file(&path);
                    }
                }
            }
        });
    }

    pub fn active_pane(&self) -> &Rc<PaneState> {
        match self.active_side.get() {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    pub fn inactive_pane(&self) -> &Rc<PaneState> {
        match self.active_side.get() {
            Side::Left => &self.right,
            Side::Right => &self.left,
        }
    }

    pub fn swap_focus(&self) {
        match self.active_side.get() {
            Side::Left => {
                self.active_side.set(Side::Right);
                self.right.container.add_css_class("pane-active");
                self.left.container.remove_css_class("pane-active");
            }
            Side::Right => {
                self.active_side.set(Side::Left);
                self.left.container.add_css_class("pane-active");
                self.right.container.remove_css_class("pane-active");
            }
        }
    }
}
