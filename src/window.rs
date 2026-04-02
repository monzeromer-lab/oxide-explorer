use adw::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;
use crate::models::file_list_model::FileListModel;
use crate::operations::{monitor::DirectoryMonitor, read_dir};
use crate::state::navigation::NavigationState;
use crate::state::selection::{ClipboardOp, ClipboardState};
use crate::utils::runtime;
use crate::widgets::breadcrumb::BreadcrumbBar;
use crate::widgets::content_view::ContentView;
use crate::widgets::filter_bar::FilterBar;
use crate::widgets::header_bar::HeaderBar;
use crate::widgets::sidebar::Sidebar;
use crate::widgets::status_bar::StatusBar;

pub struct OxideWindow {
    pub window: adw::ApplicationWindow,
}

impl OxideWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Oxide Explorer")
            .default_width(1100)
            .default_height(700)
            .build();

        let nav_state = Rc::new(RefCell::new(NavigationState::new(glib::home_dir())));
        let file_model = FileListModel::new();
        let clipboard: Rc<RefCell<Option<ClipboardState>>> = Rc::new(RefCell::new(None));
        let monitor_holder: Rc<RefCell<Option<DirectoryMonitor>>> = Rc::new(RefCell::new(None));
        let show_hidden = Rc::new(Cell::new(false));

        // We use a FilterListModel to handle hidden files and text filtering
        let filter = gtk::CustomFilter::new(|_| true);
        let filter_model = gtk::FilterListModel::new(Some(file_model.clone()), Some(filter.clone()));
        let selection_model = gtk::MultiSelection::new(Some(filter_model.clone()));

        // Build widgets
        let header = HeaderBar::new();
        let content = Rc::new(ContentView::new(&selection_model));
        let status_bar = Rc::new(StatusBar::new());
        let filter_bar = Rc::new(FilterBar::new());

        // Breadcrumb
        let breadcrumb = Rc::new(BreadcrumbBar::new(|_| {}));
        header.breadcrumb_box.append(&breadcrumb.widget);

        // --- Filter text for the custom filter ---
        let filter_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        // Wire filter bar search-changed → update filter
        let ft = filter_text.clone();
        let filter_ref = filter.clone();
        filter_bar.entry.connect_search_changed(move |entry| {
            let text = entry.text().to_string().to_lowercase();
            *ft.borrow_mut() = text;
            filter_ref.changed(gtk::FilterChange::Different);
        });

        // Set up the actual filter function
        let show_hidden_ref = show_hidden.clone();
        let filter_text_ref = filter_text.clone();
        filter.set_filter_func(move |item| {
            let entry = item.downcast_ref::<FileEntry>().unwrap();
            let name = entry.name();

            // Hidden files filter
            if !show_hidden_ref.get() && name.starts_with('.') {
                return false;
            }

            // Text filter
            let ft = filter_text_ref.borrow();
            if ft.is_empty() {
                return true;
            }
            name.to_lowercase().contains(ft.as_str())
        });

        // --- Main load_directory closure ---
        let nav_ref = nav_state.clone();
        let model_ref = file_model.clone();
        let breadcrumb_ref = breadcrumb.clone();
        let header_back = header.back_btn.clone();
        let header_fwd = header.forward_btn.clone();
        let status_ref = status_bar.clone();
        let monitor_ref = monitor_holder.clone();
        let content_ref = content.clone();
        let filter_ref2 = filter.clone();
        let filter_model_ref = filter_model.clone();

        let load_directory: Rc<dyn Fn(PathBuf)> = Rc::new(move |path: PathBuf| {
            let model = model_ref.clone();
            let breadcrumb = breadcrumb_ref.clone();
            let back_btn = header_back.clone();
            let fwd_btn = header_fwd.clone();
            let nav = nav_ref.clone();
            let status = status_ref.clone();
            let monitor = monitor_ref.clone();
            let content = content_ref.clone();
            let filter_ref = filter_ref2.clone();
            let filter_model = filter_model_ref.clone();

            // Show loading
            content.show_loading();

            // Update breadcrumb
            breadcrumb.set_path(&path, {
                let model = model.clone();
                let back_btn = back_btn.clone();
                let fwd_btn = fwd_btn.clone();
                let nav = nav.clone();
                let status = status.clone();
                let monitor = monitor.clone();
                let content = content.clone();
                let filter_ref = filter_ref.clone();
                let filter_model = filter_model.clone();
                move |target| {
                    {
                        let mut n = nav.borrow_mut();
                        n.navigate_to(target.clone());
                        back_btn.set_sensitive(n.can_go_back());
                        fwd_btn.set_sensitive(n.can_go_forward());
                    }
                    content.show_loading();
                    load_path_async(
                        target,
                        model.clone(),
                        status.clone(),
                        monitor.clone(),
                        content.clone(),
                        filter_ref.clone(),
                        filter_model.clone(),
                    );
                }
            });

            // Update nav button state
            {
                let n = nav.borrow();
                back_btn.set_sensitive(n.can_go_back());
                fwd_btn.set_sensitive(n.can_go_forward());
            }

            load_path_async(path, model, status, monitor, content, filter_ref, filter_model);
        });

        // --- Sidebar ---
        let load_for_sidebar = load_directory.clone();
        let nav_for_sidebar = nav_state.clone();
        let sidebar = Sidebar::new(move |path| {
            nav_for_sidebar.borrow_mut().navigate_to(path.clone());
            (load_for_sidebar)(path);
        });

        // --- Navigation buttons ---
        let load_for_back = load_directory.clone();
        let nav_for_back = nav_state.clone();
        header.back_btn.connect_clicked(move |_| {
            let path = {
                let mut n = nav_for_back.borrow_mut();
                if n.go_back() { Some(n.current.clone()) } else { None }
            };
            if let Some(p) = path { (load_for_back)(p); }
        });

        let load_for_fwd = load_directory.clone();
        let nav_for_fwd = nav_state.clone();
        header.forward_btn.connect_clicked(move |_| {
            let path = {
                let mut n = nav_for_fwd.borrow_mut();
                if n.go_forward() { Some(n.current.clone()) } else { None }
            };
            if let Some(p) = path { (load_for_fwd)(p); }
        });

        let load_for_up = load_directory.clone();
        let nav_for_up = nav_state.clone();
        header.up_btn.connect_clicked(move |_| {
            let path = {
                let mut n = nav_for_up.borrow_mut();
                if n.go_up() { Some(n.current.clone()) } else { None }
            };
            if let Some(p) = path { (load_for_up)(p); }
        });

        // --- View toggle ---
        let content_for_toggle = content.clone();
        header.view_toggle.connect_toggled(move |btn| {
            let is_details = content_for_toggle.toggle_view();
            if is_details {
                btn.set_icon_name("view-grid-symbolic");
                btn.set_tooltip_text(Some("Toggle Icon View"));
            } else {
                btn.set_icon_name("view-list-symbolic");
                btn.set_tooltip_text(Some("Toggle Details View"));
            }
        });

        // --- Hidden files toggle ---
        let show_hidden_for_toggle = show_hidden.clone();
        let filter_for_hidden = filter.clone();
        header.hidden_toggle.connect_toggled(move |btn| {
            show_hidden_for_toggle.set(btn.is_active());
            filter_for_hidden.changed(gtk::FilterChange::Different);
        });

        // --- Search/filter toggle ---
        let filter_bar_for_search = filter_bar.clone();
        header.search_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                filter_bar_for_search.show();
            } else {
                filter_bar_for_search.hide();
            }
        });

        // --- Double-click / activate to open ---
        let load_for_icon = load_directory.clone();
        let nav_for_icon = nav_state.clone();
        let sel_icon = selection_model.clone();
        content.icon_view.grid_view.connect_activate(move |_, pos| {
            handle_activate(&sel_icon, pos, &nav_for_icon, &load_for_icon);
        });

        let load_for_details = load_directory.clone();
        let nav_for_details = nav_state.clone();
        let sel_details = selection_model.clone();
        content.details_view.column_view.connect_activate(move |_, pos| {
            handle_activate(&sel_details, pos, &nav_for_details, &load_for_details);
        });

        // --- Selection tracking for status bar ---
        let status_for_sel = status_bar.clone();
        selection_model.connect_selection_changed(move |sel, _, _| {
            let mut count = 0u32;
            let n = sel.n_items();
            for i in 0..n {
                if sel.is_selected(i) {
                    count += 1;
                }
            }
            status_for_sel.set_selection_info(count);
        });

        // --- Context menu ---
        let context_menu = build_context_menu();

        // --- Layout ---
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.append(&filter_bar.widget);
        content_box.append(&content.outer_stack);
        content_box.append(&status_bar.widget);

        // Attach context menu to outer_stack
        context_menu.set_parent(&content.outer_stack);

        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        let menu_ref = context_menu.clone();
        gesture.connect_pressed(move |_gesture, _, x, y| {
            menu_ref.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            menu_ref.popup();
        });
        content.outer_stack.add_controller(gesture);

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_position(200);
        paned.set_start_child(Some(&sidebar.widget));
        paned.set_end_child(Some(&content_box));
        paned.set_shrink_start_child(false);
        paned.set_resize_start_child(false);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header.widget);
        toolbar_view.set_content(Some(&paned));

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&toolbar_view));

        window.set_content(Some(&toast_overlay));

        // --- Actions & Keyboard shortcuts ---
        setup_actions(
            &window,
            &nav_state,
            &selection_model,
            &clipboard,
            &load_directory,
            &toast_overlay,
            &breadcrumb,
            &filter_bar,
            &show_hidden,
            &filter,
            &header,
        );

        // Initial directory load
        (load_directory)(glib::home_dir());

        Self { window }
    }
}

fn handle_activate(
    selection: &gtk::MultiSelection,
    pos: u32,
    nav_state: &Rc<RefCell<NavigationState>>,
    load_directory: &Rc<dyn Fn(PathBuf)>,
) {
    if let Some(item) = selection.item(pos) {
        if let Some(entry) = item.downcast_ref::<FileEntry>() {
            let path = PathBuf::from(entry.path());
            if entry.is_dir() {
                nav_state.borrow_mut().navigate_to(path.clone());
                (load_directory)(path);
            } else {
                // Open file with default application
                open_file(&path);
            }
        }
    }
}

fn open_file(path: &PathBuf) {
    let file = gio::File::for_path(path);
    let launcher = gtk::FileLauncher::new(Some(&file));
    // Launch without a parent window — fire and forget
    launcher.launch(gtk::Window::NONE, gio::Cancellable::NONE, |result| {
        if let Err(e) = result {
            log::warn!("Failed to open file: {e}");
        }
    });
}

fn load_path_async(
    path: PathBuf,
    model: FileListModel,
    status: Rc<StatusBar>,
    monitor: Rc<RefCell<Option<DirectoryMonitor>>>,
    content: Rc<ContentView>,
    filter: gtk::CustomFilter,
    filter_model: gtk::FilterListModel,
) {
    let model_for_monitor = model.clone();
    let status_for_monitor = status.clone();
    let content_for_monitor = content.clone();
    let filter_for_monitor = filter.clone();
    let filter_model_for_monitor = filter_model.clone();
    let path_for_monitor = path.clone();

    glib::spawn_future_local(async move {
        let result = runtime::spawn(async move {
            read_dir::read_directory(&path).await
        })
        .await;

        match result {
            Ok(Ok(entries)) => {
                let file_entries: Vec<FileEntry> = entries
                    .into_iter()
                    .map(|e| {
                        FileEntry::new(
                            &e.name,
                            &e.path.to_string_lossy(),
                            e.size,
                            e.modified,
                            e.is_dir,
                            &e.icon_name,
                            &e.content_type,
                        )
                    })
                    .collect();
                let total = file_entries.len() as u32;
                model.replace_all(file_entries);
                // Re-evaluate filter
                filter.changed(gtk::FilterChange::Different);
                let visible = filter_model.n_items();
                status.set_item_count(visible, total);
                content.show_content(visible);
            }
            Ok(Err(e)) => {
                log::error!("Failed to read directory: {e}");
                model.clear();
                content.show_error(&e.to_string());
                status.set_item_count(0, 0);
            }
            Err(e) => {
                log::error!("Task failed: {e}");
                model.clear();
                content.show_error(&e.to_string());
                status.set_item_count(0, 0);
            }
        }
    });

    // Set up file monitor for auto-refresh (debounced)
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let path_for_cb = path_for_monitor.clone();
    let m = DirectoryMonitor::new(&path_for_monitor, move || {
        // Debounce: cancel previous pending reload, schedule a new one in 300ms
        let mut src = debounce_source.borrow_mut();
        if let Some(old) = src.take() {
            old.remove();
        }
        let model = model_for_monitor.clone();
        let status = status_for_monitor.clone();
        let content = content_for_monitor.clone();
        let filter = filter_for_monitor.clone();
        let filter_model = filter_model_for_monitor.clone();
        let path = path_for_cb.clone();
        let monitor_inner: Rc<RefCell<Option<DirectoryMonitor>>> = Rc::new(RefCell::new(None));
        *src = Some(glib::timeout_add_local_once(
            std::time::Duration::from_millis(300),
            move || {
                load_path_async(path, model, status, monitor_inner, content, filter, filter_model);
            },
        ));
    });
    *monitor.borrow_mut() = m;
}

fn build_context_menu() -> gtk::PopoverMenu {
    let menu = gio::Menu::new();

    let file_section = gio::Menu::new();
    file_section.append(Some("New Folder"), Some("win.new-folder"));
    menu.append_section(None, &file_section);

    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy"), Some("win.copy"));
    edit_section.append(Some("Cut"), Some("win.cut"));
    edit_section.append(Some("Paste"), Some("win.paste"));
    edit_section.append(Some("Rename"), Some("win.rename"));
    menu.append_section(None, &edit_section);

    let delete_section = gio::Menu::new();
    delete_section.append(Some("Move to Trash"), Some("win.trash"));
    menu.append_section(None, &delete_section);

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover
}

#[allow(clippy::too_many_arguments)]
fn setup_actions(
    window: &adw::ApplicationWindow,
    nav_state: &Rc<RefCell<NavigationState>>,
    selection_model: &gtk::MultiSelection,
    clipboard: &Rc<RefCell<Option<ClipboardState>>>,
    load_directory: &Rc<dyn Fn(PathBuf)>,
    toast_overlay: &adw::ToastOverlay,
    breadcrumb: &Rc<BreadcrumbBar>,
    filter_bar: &Rc<FilterBar>,
    show_hidden: &Rc<Cell<bool>>,
    filter: &gtk::CustomFilter,
    header: &HeaderBar,
) {
    // --- Copy ---
    let clip = clipboard.clone();
    let sel = selection_model.clone();
    let toast = toast_overlay.clone();
    let copy_action = gio::SimpleAction::new("copy", None);
    copy_action.connect_activate(move |_, _| {
        let paths = get_selected_paths(&sel);
        if !paths.is_empty() {
            let count = paths.len();
            *clip.borrow_mut() = Some(ClipboardState {
                operation: ClipboardOp::Copy,
                paths,
            });
            toast.add_toast(adw::Toast::new(&format!("{count} item(s) copied")));
        }
    });
    window.add_action(&copy_action);

    // --- Cut ---
    let clip = clipboard.clone();
    let sel = selection_model.clone();
    let toast = toast_overlay.clone();
    let cut_action = gio::SimpleAction::new("cut", None);
    cut_action.connect_activate(move |_, _| {
        let paths = get_selected_paths(&sel);
        if !paths.is_empty() {
            let count = paths.len();
            *clip.borrow_mut() = Some(ClipboardState {
                operation: ClipboardOp::Cut,
                paths,
            });
            toast.add_toast(adw::Toast::new(&format!("{count} item(s) cut")));
        }
    });
    window.add_action(&cut_action);

    // --- Paste ---
    let clip = clipboard.clone();
    let nav = nav_state.clone();
    let load = load_directory.clone();
    let toast = toast_overlay.clone();
    let paste_action = gio::SimpleAction::new("paste", None);
    paste_action.connect_activate(move |_, _| {
        let state = clip.borrow().clone();
        if let Some(state) = state {
            let dest_dir = nav.borrow().current.clone();
            let load = load.clone();
            let toast = toast.clone();
            let clip = clip.clone();
            let is_cut = state.operation == ClipboardOp::Cut;

            for source in &state.paths {
                let file_name = source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let dest = dest_dir.join(&file_name);
                let src = source.clone();
                let op = state.operation;
                let load = load.clone();
                let toast = toast.clone();
                let dest_dir = dest_dir.clone();

                glib::spawn_future_local(async move {
                    let result = runtime::spawn(async move {
                        if op == ClipboardOp::Copy {
                            crate::operations::file_ops::copy_file(&src, &dest).await
                        } else {
                            crate::operations::file_ops::move_file(&src, &dest).await
                        }
                    })
                    .await;

                    match result {
                        Ok(Ok(())) => {
                            toast.add_toast(adw::Toast::new("Paste complete"));
                            (load)(dest_dir);
                        }
                        Ok(Err(e)) => {
                            toast.add_toast(adw::Toast::new(&format!("Paste failed: {e}")));
                        }
                        Err(e) => {
                            toast.add_toast(adw::Toast::new(&format!("Paste failed: {e}")));
                        }
                    }
                });
            }

            if is_cut {
                *clip.borrow_mut() = None;
            }
        }
    });
    window.add_action(&paste_action);

    // --- Trash (with confirmation) ---
    let sel = selection_model.clone();
    let nav = nav_state.clone();
    let load = load_directory.clone();
    let toast = toast_overlay.clone();
    let win = window.clone();
    let trash_action = gio::SimpleAction::new("trash", None);
    trash_action.connect_activate(move |_, _| {
        let paths = get_selected_paths(&sel);
        if paths.is_empty() {
            return;
        }

        let count = paths.len();
        let message = if count == 1 {
            let name = paths[0]
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            format!("Move \"{name}\" to trash?")
        } else {
            format!("Move {count} items to trash?")
        };

        let dialog = adw::MessageDialog::new(
            Some(&win),
            Some("Move to Trash"),
            Some(&message),
        );
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("trash", "Move to Trash");
        dialog.set_response_appearance("trash", adw::ResponseAppearance::Destructive);

        let nav = nav.clone();
        let load = load.clone();
        let toast = toast.clone();
        dialog.connect_response(None, move |dlg, response| {
            if response == "trash" {
                let dest_dir = nav.borrow().current.clone();
                for path in &paths {
                    match crate::operations::trash_ops::move_to_trash(path) {
                        Ok(()) => {}
                        Err(e) => {
                            toast.add_toast(adw::Toast::new(&format!("Trash failed: {e}")));
                        }
                    }
                }
                (load)(dest_dir);
            }
            dlg.close();
        });

        dialog.present();
    });
    window.add_action(&trash_action);

    // --- New Folder ---
    let nav = nav_state.clone();
    let load = load_directory.clone();
    let toast = toast_overlay.clone();
    let win = window.clone();
    let new_folder_action = gio::SimpleAction::new("new-folder", None);
    new_folder_action.connect_activate(move |_, _| {
        let dir = nav.borrow().current.clone();
        let load = load.clone();
        let toast = toast.clone();

        let dialog = adw::MessageDialog::new(Some(&win), Some("New Folder"), Some("Enter folder name:"));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("create", "Create");
        dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);

        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("Folder name"));
        dialog.set_extra_child(Some(&entry));

        let entry_ref = entry.clone();
        dialog.connect_response(None, move |dlg, response| {
            if response == "create" {
                let name = entry_ref.text().to_string();
                if !name.is_empty() {
                    let path = dir.join(&name);
                    let load = load.clone();
                    let toast = toast.clone();
                    let dir = dir.clone();

                    glib::spawn_future_local(async move {
                        let result = runtime::spawn(async move {
                            crate::operations::file_ops::create_directory(&path).await
                        })
                        .await;

                        match result {
                            Ok(Ok(())) => (load)(dir),
                            Ok(Err(e)) => {
                                toast.add_toast(adw::Toast::new(&format!("Create folder failed: {e}")));
                            }
                            Err(e) => {
                                toast.add_toast(adw::Toast::new(&format!("Create folder failed: {e}")));
                            }
                        }
                    });
                }
            }
            dlg.close();
        });

        dialog.present();
    });
    window.add_action(&new_folder_action);

    // --- Rename ---
    let sel = selection_model.clone();
    let load = load_directory.clone();
    let toast = toast_overlay.clone();
    let win = window.clone();
    let rename_action = gio::SimpleAction::new("rename", None);
    rename_action.connect_activate(move |_, _| {
        let paths = get_selected_paths(&sel);
        if let Some(old_path) = paths.first() {
            let old_name = old_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let parent = old_path.parent().unwrap_or(old_path).to_path_buf();
            let old_path = old_path.clone();
            let load = load.clone();
            let toast = toast.clone();

            let dialog = adw::MessageDialog::new(
                Some(&win),
                Some("Rename"),
                Some(&format!("Rename \"{old_name}\":")),
            );
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("rename", "Rename");
            dialog.set_response_appearance("rename", adw::ResponseAppearance::Suggested);

            let entry = gtk::Entry::new();
            entry.set_text(&old_name);
            dialog.set_extra_child(Some(&entry));

            let entry_ref = entry.clone();
            let old_name = old_name.clone();
            dialog.connect_response(None, move |dlg, response| {
                if response == "rename" {
                    let new_name = entry_ref.text().to_string();
                    if !new_name.is_empty() && new_name != old_name {
                        let new_path = parent.join(&new_name);
                        let from = old_path.clone();
                        let load = load.clone();
                        let toast = toast.clone();
                        let parent = parent.clone();

                        glib::spawn_future_local(async move {
                            let result = runtime::spawn(async move {
                                crate::operations::file_ops::rename(&from, &new_path).await
                            })
                            .await;

                            match result {
                                Ok(Ok(())) => (load)(parent),
                                Ok(Err(e)) => {
                                    toast.add_toast(adw::Toast::new(&format!("Rename failed: {e}")));
                                }
                                Err(e) => {
                                    toast.add_toast(adw::Toast::new(&format!("Rename failed: {e}")));
                                }
                            }
                        });
                    }
                }
                dlg.close();
            });

            dialog.present();
        }
    });
    window.add_action(&rename_action);

    // --- Toggle hidden files action ---
    let hidden = show_hidden.clone();
    let filter_a = filter.clone();
    let toggle_btn = header.hidden_toggle.clone();
    let toggle_hidden_action = gio::SimpleAction::new("toggle-hidden", None);
    toggle_hidden_action.connect_activate(move |_, _| {
        let new_val = !hidden.get();
        hidden.set(new_val);
        toggle_btn.set_active(new_val);
        filter_a.changed(gtk::FilterChange::Different);
    });
    window.add_action(&toggle_hidden_action);

    // --- Location bar action (Ctrl+L) ---
    let bc = breadcrumb.clone();
    let location_action = gio::SimpleAction::new("location-bar", None);
    location_action.connect_activate(move |_, _| {
        bc.enter_edit_mode();
    });
    window.add_action(&location_action);

    // --- Filter/search action (Ctrl+F) ---
    let fb = filter_bar.clone();
    let search_btn = header.search_btn.clone();
    let filter_action = gio::SimpleAction::new("toggle-filter", None);
    filter_action.connect_activate(move |_, _| {
        fb.toggle();
        search_btn.set_active(fb.is_visible());
    });
    window.add_action(&filter_action);

    // --- Navigate back/forward/up actions for keyboard ---
    let load = load_directory.clone();
    let nav = nav_state.clone();
    let go_back_action = gio::SimpleAction::new("go-back", None);
    go_back_action.connect_activate(move |_, _| {
        let path = {
            let mut n = nav.borrow_mut();
            if n.go_back() { Some(n.current.clone()) } else { None }
        };
        if let Some(p) = path { (load)(p); }
    });
    window.add_action(&go_back_action);

    let load = load_directory.clone();
    let nav = nav_state.clone();
    let go_forward_action = gio::SimpleAction::new("go-forward", None);
    go_forward_action.connect_activate(move |_, _| {
        let path = {
            let mut n = nav.borrow_mut();
            if n.go_forward() { Some(n.current.clone()) } else { None }
        };
        if let Some(p) = path { (load)(p); }
    });
    window.add_action(&go_forward_action);

    let load = load_directory.clone();
    let nav = nav_state.clone();
    let go_up_action = gio::SimpleAction::new("go-up", None);
    go_up_action.connect_activate(move |_, _| {
        let path = {
            let mut n = nav.borrow_mut();
            if n.go_up() { Some(n.current.clone()) } else { None }
        };
        if let Some(p) = path { (load)(p); }
    });
    window.add_action(&go_up_action);

    // --- Keyboard shortcuts ---
    let shortcut_controller = gtk::ShortcutController::new();
    shortcut_controller.set_scope(gtk::ShortcutScope::Managed);

    let shortcuts = [
        ("<Control>c", "win.copy"),
        ("<Control>x", "win.cut"),
        ("<Control>v", "win.paste"),
        ("Delete", "win.trash"),
        ("F2", "win.rename"),
        ("<Control>h", "win.toggle-hidden"),
        ("<Control>l", "win.location-bar"),
        ("<Control>f", "win.toggle-filter"),
        ("<Alt>Left", "win.go-back"),
        ("<Alt>Right", "win.go-forward"),
        ("<Alt>Up", "win.go-up"),
        ("BackSpace", "win.go-back"),
    ];

    for (key, action) in shortcuts {
        let shortcut = gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string(key),
            Some(gtk::NamedAction::new(action)),
        );
        shortcut_controller.add_shortcut(shortcut);
    }

    window.add_controller(shortcut_controller);
}

fn get_selected_paths(selection: &gtk::MultiSelection) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let n = selection.n_items();
    for i in 0..n {
        if selection.is_selected(i) {
            if let Some(item) = selection.item(i) {
                if let Some(entry) = item.downcast_ref::<FileEntry>() {
                    paths.push(PathBuf::from(entry.path()));
                }
            }
        }
    }
    paths
}

impl Clone for ClipboardState {
    fn clone(&self) -> Self {
        Self {
            operation: self.operation,
            paths: self.paths.clone(),
        }
    }
}
