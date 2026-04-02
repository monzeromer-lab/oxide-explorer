use adw::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::file_entry::FileEntry;
use crate::models::file_list_model::FileListModel;
use crate::operations::{monitor::DirectoryMonitor, read_dir};
use crate::state::navigation::NavigationState;
use crate::state::selection::{ClipboardOp, ClipboardState};
use crate::state::settings::Settings;
use crate::state::tab::TabState;
use crate::utils::runtime;
use crate::state::keybindings::KeybindingConfig;
use crate::widgets::breadcrumb::BreadcrumbBar;
use crate::widgets::content_view::ContentView;
use crate::widgets::dual_pane::DualPane;
use crate::widgets::header_bar::HeaderBar;
use crate::widgets::miller_columns::MillerColumnsView;
use crate::widgets::sidebar::Sidebar;
use crate::widgets::status_bar::StatusBar;
use crate::widgets::terminal_panel::TerminalPanel;

pub struct OxideWindow {
    pub window: adw::ApplicationWindow,
}

impl OxideWindow {
    pub fn new(app: &adw::Application) -> Self {
        let settings = Rc::new(RefCell::new(Settings::load()));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Oxide Explorer")
            .default_width(1100)
            .default_height(700)
            .build();

        let clipboard: Rc<RefCell<Option<ClipboardState>>> = Rc::new(RefCell::new(None));
        let show_hidden = Rc::new(Cell::new(settings.borrow().show_hidden_files));
        let icon_size = Rc::new(Cell::new(settings.borrow().icon_size));

        // --- Tab infrastructure ---
        let tab_view = adw::TabView::new();
        let tab_bar = adw::TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        tab_bar.set_autohide(true); // Hide when only 1 tab

        // Store all tab states keyed by their TabPage
        let tabs: Rc<RefCell<Vec<Rc<TabState>>>> = Rc::new(RefCell::new(Vec::new()));

        // Header, breadcrumb, sidebar
        let header = HeaderBar::new();
        let breadcrumb = Rc::new(BreadcrumbBar::new(|_| {}));
        header.breadcrumb_box.append(&breadcrumb.widget);

        if show_hidden.get() {
            header.hidden_toggle.set_active(true);
        }

        // --- Helper: get active tab ---
        let get_active_tab: ActiveTabFn = {
            let tab_view = tab_view.clone();
            let tabs = tabs.clone();
            Rc::new(move || -> Option<Rc<TabState>> {
                let page = tab_view.selected_page()?;
                let child = page.child();
                let tabs = tabs.borrow();
                tabs.iter()
                    .find(|t| t.container == child)
                    .cloned()
            })
        };

        // --- Create a load_directory for a specific tab ---
        fn make_load_for_tab(
            tab: &Rc<TabState>,
            breadcrumb: &Rc<BreadcrumbBar>,
            back_btn: &gtk::Button,
            fwd_btn: &gtk::Button,
            tab_view: &adw::TabView,
        ) -> Rc<dyn Fn(PathBuf)> {
            let tab = tab.clone();
            let breadcrumb = breadcrumb.clone();
            let back_btn = back_btn.clone();
            let fwd_btn = fwd_btn.clone();
            let tab_view = tab_view.clone();

            Rc::new(move |path: PathBuf| {
                let tab = tab.clone();
                let breadcrumb = breadcrumb.clone();
                let back_btn = back_btn.clone();
                let fwd_btn = fwd_btn.clone();
                let tab_view = tab_view.clone();

                tab.content.show_loading();

                // Update breadcrumb
                breadcrumb.set_path(&path, {
                    let tab = tab.clone();
                    let back_btn = back_btn.clone();
                    let fwd_btn = fwd_btn.clone();
                    move |target| {
                        {
                            let mut n = tab.nav.borrow_mut();
                            n.navigate_to(target.clone());
                            back_btn.set_sensitive(n.can_go_back());
                            fwd_btn.set_sensitive(n.can_go_forward());
                        }
                        tab.content.show_loading();
                        load_path_async(
                            target,
                            tab.file_model.clone(),
                            tab.status_bar.clone(),
                            tab.monitor.clone(),
                            tab.content.clone(),
                            tab.filter.clone(),
                            tab.filter_model.clone(),
                        );
                    }
                });

                {
                    let n = tab.nav.borrow();
                    back_btn.set_sensitive(n.can_go_back());
                    fwd_btn.set_sensitive(n.can_go_forward());
                }

                // Update tab title
                let title = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".to_string());
                tab_view.page(&tab.container).set_title(&title);

                load_path_async(
                    path,
                    tab.file_model.clone(),
                    tab.status_bar.clone(),
                    tab.monitor.clone(),
                    tab.content.clone(),
                    tab.filter.clone(),
                    tab.filter_model.clone(),
                );
            })
        }

        // --- Create new tab function ---
        let create_tab: CreateTabFn = {
            let tabs = tabs.clone();
            let tab_view = tab_view.clone();
            let show_hidden = show_hidden.clone();
            let icon_size = icon_size.clone();
            let breadcrumb = breadcrumb.clone();
            let back_btn = header.back_btn.clone();
            let fwd_btn = header.forward_btn.clone();

            Rc::new(move |path: PathBuf| -> Rc<TabState> {
                let tab = Rc::new(TabState::new(
                    path.clone(),
                    show_hidden.clone(),
                    icon_size.clone(),
                ));

                let load = make_load_for_tab(&tab, &breadcrumb, &back_btn, &fwd_btn, &tab_view);

                // Wire double-click / activate on icon view
                let load_icon = load.clone();
                let nav_icon = tab.nav.clone();
                let sel_icon = tab.selection_model.clone();
                tab.content.icon_view.grid_view.connect_activate(move |_, pos| {
                    handle_activate(&sel_icon, pos, &nav_icon, &load_icon);
                });

                // Wire activate on details view
                let load_det = load.clone();
                let nav_det = tab.nav.clone();
                let sel_det = tab.selection_model.clone();
                tab.content.details_view.column_view.connect_activate(move |_, pos| {
                    handle_activate(&sel_det, pos, &nav_det, &load_det);
                });

                // Context menu for this tab
                let context_menu = build_context_menu();
                context_menu.set_parent(&tab.content.outer_stack);
                let menu_ref = context_menu.clone();
                let gesture = gtk::GestureClick::new();
                gesture.set_button(3);
                gesture.connect_pressed(move |_, _, x, y| {
                    menu_ref.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                    menu_ref.popup();
                });
                tab.content.outer_stack.add_controller(gesture);

                // Drop target for this tab
                let drop_target = gtk::DropTarget::new(gio::File::static_type(), gtk::gdk::DragAction::COPY | gtk::gdk::DragAction::MOVE);
                let nav_drop = tab.nav.clone();
                let load_drop = load.clone();
                drop_target.connect_drop(move |_, value, _, _| {
                    if let Ok(file) = value.get::<gio::File>() {
                        if let Some(source_path) = file.path() {
                            let dest_dir = nav_drop.borrow().current.clone();
                            let file_name = source_path.file_name().unwrap_or_default().to_os_string();
                            let dest = dest_dir.join(&file_name);
                            let load = load_drop.clone();
                            glib::spawn_future_local(async move {
                                let result = runtime::spawn(async move {
                                    crate::operations::file_ops::move_file(&source_path, &dest).await
                                }).await;
                                if let Ok(Ok(())) = result { (load)(dest_dir); }
                            });
                            return true;
                        }
                    }
                    false
                });
                tab.content.outer_stack.add_controller(drop_target);

                // Drag source
                let drag_source = gtk::DragSource::new();
                drag_source.set_actions(gtk::gdk::DragAction::COPY | gtk::gdk::DragAction::MOVE);
                let sel_drag = tab.selection_model.clone();
                drag_source.connect_prepare(move |_, _, _| {
                    let paths = get_selected_paths(&sel_drag);
                    paths.first().map(|p| {
                        let file = gio::File::for_path(p);
                        gtk::gdk::ContentProvider::for_value(&file.to_value())
                    })
                });
                tab.content.icon_view.grid_view.add_controller(drag_source);

                // Add page to tab view
                let title = path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".to_string());
                let page = tab_view.append(&tab.container);
                page.set_title(&title);

                tabs.borrow_mut().push(tab.clone());

                // Load initial directory
                (load)(path);

                tab
            })
        };

        // --- Sidebar ---
        let tab_view_for_sidebar = tab_view.clone();
        let breadcrumb_for_sidebar = breadcrumb.clone();
        let back_btn_for_sidebar = header.back_btn.clone();
        let fwd_btn_for_sidebar = header.forward_btn.clone();
        let get_active_for_sidebar = get_active_tab.clone();
        let sidebar = Rc::new(Sidebar::new(move |path| {
            // Navigate the active tab
            if let Some(tab) = get_active_for_sidebar() {
                let load = make_load_for_tab(&tab, &breadcrumb_for_sidebar, &back_btn_for_sidebar, &fwd_btn_for_sidebar, &tab_view_for_sidebar);
                tab.nav.borrow_mut().navigate_to(path.clone());
                (load)(path);
            }
        }));

        // --- Navigation buttons ---
        let get_active_back = get_active_tab.clone();
        let breadcrumb_back = breadcrumb.clone();
        let back_btn_ref = header.back_btn.clone();
        let fwd_btn_ref = header.forward_btn.clone();
        let tab_view_back = tab_view.clone();
        header.back_btn.connect_clicked(move |_| {
            if let Some(tab) = get_active_back() {
                let path = { let mut n = tab.nav.borrow_mut(); if n.go_back() { Some(n.current.clone()) } else { None } };
                if let Some(p) = path {
                    let load = make_load_for_tab(&tab, &breadcrumb_back, &back_btn_ref, &fwd_btn_ref, &tab_view_back);
                    (load)(p);
                }
            }
        });

        let get_active_fwd = get_active_tab.clone();
        let breadcrumb_fwd = breadcrumb.clone();
        let back_btn_fwd = header.back_btn.clone();
        let fwd_btn_fwd = header.forward_btn.clone();
        let tab_view_fwd = tab_view.clone();
        header.forward_btn.connect_clicked(move |_| {
            if let Some(tab) = get_active_fwd() {
                let path = { let mut n = tab.nav.borrow_mut(); if n.go_forward() { Some(n.current.clone()) } else { None } };
                if let Some(p) = path {
                    let load = make_load_for_tab(&tab, &breadcrumb_fwd, &back_btn_fwd, &fwd_btn_fwd, &tab_view_fwd);
                    (load)(p);
                }
            }
        });

        let get_active_up = get_active_tab.clone();
        let breadcrumb_up = breadcrumb.clone();
        let back_btn_up = header.back_btn.clone();
        let fwd_btn_up = header.forward_btn.clone();
        let tab_view_up = tab_view.clone();
        header.up_btn.connect_clicked(move |_| {
            if let Some(tab) = get_active_up() {
                let path = { let mut n = tab.nav.borrow_mut(); if n.go_up() { Some(n.current.clone()) } else { None } };
                if let Some(p) = path {
                    let load = make_load_for_tab(&tab, &breadcrumb_up, &back_btn_up, &fwd_btn_up, &tab_view_up);
                    (load)(p);
                }
            }
        });

        // --- View toggle ---
        let get_active_view = get_active_tab.clone();
        header.view_toggle.connect_toggled(move |btn| {
            if let Some(tab) = get_active_view() {
                let is_details = tab.content.toggle_view();
                if is_details {
                    btn.set_icon_name("view-grid-symbolic");
                    btn.set_tooltip_text(Some("Toggle Icon View"));
                } else {
                    btn.set_icon_name("view-list-symbolic");
                    btn.set_tooltip_text(Some("Toggle Details View"));
                }
            }
        });

        // --- Hidden files toggle ---
        let sh = show_hidden.clone();
        let tabs_hidden = tabs.clone();
        header.hidden_toggle.connect_toggled(move |btn| {
            sh.set(btn.is_active());
            // Update filter on ALL tabs
            for tab in tabs_hidden.borrow().iter() {
                tab.filter.changed(gtk::FilterChange::Different);
            }
        });

        // --- Search/filter toggle ---
        let get_active_search = get_active_tab.clone();
        header.search_btn.connect_toggled(move |btn| {
            if let Some(tab) = get_active_search() {
                if btn.is_active() { tab.filter_bar.show(); } else { tab.filter_bar.hide(); }
            }
        });

        // --- Zoom ---
        let icon_size_in = icon_size.clone();
        let settings_in = settings.clone();
        header.zoom_in_btn.connect_clicked(move |_| {
            let new = (icon_size_in.get() + 8).min(128);
            icon_size_in.set(new);
            settings_in.borrow_mut().icon_size = new;
            settings_in.borrow().save();
        });
        let icon_size_out = icon_size.clone();
        let settings_out = settings.clone();
        header.zoom_out_btn.connect_clicked(move |_| {
            let new = (icon_size_out.get() - 8).max(24);
            icon_size_out.set(new);
            settings_out.borrow_mut().icon_size = new;
            settings_out.borrow().save();
        });

        // --- Tab switching: update breadcrumb + nav buttons ---
        let breadcrumb_switch = breadcrumb.clone();
        let back_btn_switch = header.back_btn.clone();
        let fwd_btn_switch = header.forward_btn.clone();
        let tabs_switch = tabs.clone();
        let tab_view_switch = tab_view.clone();
        tab_view.connect_selected_page_notify(move |tv| {
            if let Some(page) = tv.selected_page() {
                let child = page.child();
                let tabs = tabs_switch.borrow();
                if let Some(tab) = tabs.iter().find(|t| t.container == child) {
                    let path = tab.current_path();
                    let n = tab.nav.borrow();
                    back_btn_switch.set_sensitive(n.can_go_back());
                    fwd_btn_switch.set_sensitive(n.can_go_forward());
                    drop(n);
                    // Update breadcrumb (set_path needs a navigate callback)
                    let tab2 = tab.clone();
                    let bb = back_btn_switch.clone();
                    let fb = fwd_btn_switch.clone();
                    breadcrumb_switch.set_path(&path, move |target| {
                        let mut n = tab2.nav.borrow_mut();
                        n.navigate_to(target.clone());
                        bb.set_sensitive(n.can_go_back());
                        fb.set_sensitive(n.can_go_forward());
                        drop(n);
                        load_path_async(
                            target,
                            tab2.file_model.clone(),
                            tab2.status_bar.clone(),
                            tab2.monitor.clone(),
                            tab2.content.clone(),
                            tab2.filter.clone(),
                            tab2.filter_model.clone(),
                        );
                    });
                }
            }
        });

        // --- Tab close ---
        let tabs_close = tabs.clone();
        tab_view.connect_close_page(move |tv, page| {
            let child = page.child();
            tabs_close.borrow_mut().retain(|t| t.container != child);
            tv.close_page_finish(page, true);
            glib::Propagation::Stop
        });

        // --- Phase 2 widgets ---
        let terminal = Rc::new(TerminalPanel::new());
        let dual_pane = Rc::new(DualPane::new(show_hidden.clone(), icon_size.clone()));
        let miller = Rc::new(MillerColumnsView::new(|path| { open_file(&path); }));

        // Miller columns revealer
        let miller_revealer = gtk::Revealer::new();
        miller_revealer.set_child(Some(&miller.widget));
        miller_revealer.set_reveal_child(false);
        miller_revealer.set_transition_type(gtk::RevealerTransitionType::Crossfade);

        // --- Layout ---
        // Main content area: tab_view + dual_pane + miller (stacked)
        let main_stack = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_stack.append(&tab_view);
        main_stack.append(&dual_pane.widget);
        main_stack.append(&miller_revealer);
        main_stack.append(&terminal.widget);

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_position(200);
        paned.set_start_child(Some(&sidebar.widget));
        paned.set_end_child(Some(&main_stack));
        paned.set_shrink_start_child(false);
        paned.set_resize_start_child(false);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header.widget);
        toolbar_view.add_top_bar(&tab_bar);
        toolbar_view.set_content(Some(&paned));

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&toolbar_view));
        window.set_content(Some(&toast_overlay));

        // --- Actions ---
        setup_actions(
            &window,
            &get_active_tab,
            &clipboard,
            &toast_overlay,
            &breadcrumb,
            &show_hidden,
            &header,
            &icon_size,
            &settings,
            &sidebar,
            &create_tab,
            &tab_view,
            &tabs,
            &terminal,
            &dual_pane,
            &miller,
            &miller_revealer,
        );

        // --- Create initial tab ---
        (create_tab)(glib::home_dir());

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
                open_file(&path);
            }
        }
    }
}

fn open_file(path: &PathBuf) {
    let file = gio::File::for_path(path);
    let launcher = gtk::FileLauncher::new(Some(&file));
    launcher.launch(gtk::Window::NONE, gio::Cancellable::NONE, |result| {
        if let Err(e) = result { log::warn!("Failed to open file: {e}"); }
    });
}

/// Public wrapper for dual_pane to call
pub fn load_path_async_pub(
    path: PathBuf,
    model: FileListModel,
    status: Rc<StatusBar>,
    monitor: Rc<RefCell<Option<DirectoryMonitor>>>,
    content: Rc<ContentView>,
    filter: gtk::CustomFilter,
    filter_model: gtk::FilterListModel,
) {
    load_path_async(path, model, status, monitor, content, filter, filter_model);
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
    let model_mon = model.clone();
    let status_mon = status.clone();
    let content_mon = content.clone();
    let filter_mon = filter.clone();
    let filter_model_mon = filter_model.clone();
    let path_mon = path.clone();
    let status_disk = status.clone();

    glib::spawn_future_local(async move {
        let result = runtime::spawn(async move { read_dir::read_directory(&path).await }).await;
        match result {
            Ok(Ok(entries)) => {
                let file_entries: Vec<FileEntry> = entries
                    .into_iter()
                    .map(|e| FileEntry::new(&e.name, &e.path.to_string_lossy(), e.size, e.modified, e.is_dir, e.is_symlink, &e.icon_name, &e.content_type))
                    .collect();
                let total = file_entries.len() as u32;
                model.replace_all(file_entries);
                filter.changed(gtk::FilterChange::Different);
                let visible = filter_model.n_items();
                status.set_item_count(visible, total);
                content.show_content(visible);
            }
            Ok(Err(e)) => { log::error!("Failed to read directory: {e}"); model.clear(); content.show_error(&e.to_string()); status.set_item_count(0, 0); }
            Err(e) => { log::error!("Task failed: {e}"); model.clear(); content.show_error(&e.to_string()); status.set_item_count(0, 0); }
        }
    });

    // Disk space
    let disk_path = path_mon.clone();
    glib::spawn_future_local(async move {
        let result = runtime::spawn(async move {
            let stat = nix::sys::statvfs::statvfs(&disk_path).ok()?;
            let total = stat.blocks() as u64 * stat.fragment_size() as u64;
            let free = stat.blocks_available() as u64 * stat.fragment_size() as u64;
            Some((free, total))
        }).await;
        if let Ok(Some((free, total))) = result { status_disk.set_disk_info(free, total); }
    });

    // Directory monitor
    let debounce: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let path_cb = path_mon.clone();
    let m = DirectoryMonitor::new(&path_mon, move || {
        let mut src = debounce.borrow_mut();
        if let Some(old) = src.take() { old.remove(); }
        let model = model_mon.clone();
        let status = status_mon.clone();
        let content = content_mon.clone();
        let filter = filter_mon.clone();
        let filter_model = filter_model_mon.clone();
        let path = path_cb.clone();
        let mon: Rc<RefCell<Option<DirectoryMonitor>>> = Rc::new(RefCell::new(None));
        *src = Some(glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            load_path_async(path, model, status, mon, content, filter, filter_model);
        }));
    });
    *monitor.borrow_mut() = m;
}

fn build_context_menu() -> gtk::PopoverMenu {
    let menu = gio::Menu::new();

    let file_section = gio::Menu::new();
    file_section.append(Some("New Folder"), Some("win.new-folder"));
    file_section.append(Some("Open Terminal Here"), Some("win.open-terminal"));
    file_section.append(Some("Open With..."), Some("win.open-with"));
    menu.append_section(None, &file_section);

    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy"), Some("win.copy"));
    edit_section.append(Some("Cut"), Some("win.cut"));
    edit_section.append(Some("Paste"), Some("win.paste"));
    edit_section.append(Some("Copy Path"), Some("win.copy-path"));
    edit_section.append(Some("Rename"), Some("win.rename"));
    edit_section.append(Some("Select All"), Some("win.select-all"));
    menu.append_section(None, &edit_section);

    let tab_section = gio::Menu::new();
    tab_section.append(Some("Open in New Tab"), Some("win.open-in-new-tab"));
    tab_section.append(Some("Bookmark This Folder"), Some("win.add-bookmark"));
    menu.append_section(None, &tab_section);

    let delete_section = gio::Menu::new();
    delete_section.append(Some("Move to Trash"), Some("win.trash"));
    menu.append_section(None, &delete_section);

    let info_section = gio::Menu::new();
    info_section.append(Some("Properties"), Some("win.properties"));
    menu.append_section(None, &info_section);

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover
}

type ActiveTabFn = Rc<dyn Fn() -> Option<Rc<TabState>>>;
type CreateTabFn = Rc<dyn Fn(PathBuf) -> Rc<TabState>>;

#[allow(clippy::too_many_arguments)]
fn setup_actions(
    window: &adw::ApplicationWindow,
    get_active_tab: &ActiveTabFn,
    clipboard: &Rc<RefCell<Option<ClipboardState>>>,
    toast_overlay: &adw::ToastOverlay,
    breadcrumb: &Rc<BreadcrumbBar>,
    show_hidden: &Rc<Cell<bool>>,
    header: &HeaderBar,
    icon_size: &Rc<Cell<i32>>,
    settings: &Rc<RefCell<Settings>>,
    sidebar: &Rc<Sidebar>,
    create_tab: &CreateTabFn,
    tab_view: &adw::TabView,
    tabs: &Rc<RefCell<Vec<Rc<TabState>>>>,
    terminal: &Rc<TerminalPanel>,
    dual_pane: &Rc<DualPane>,
    miller: &Rc<MillerColumnsView>,
    miller_revealer: &gtk::Revealer,
) {
    // Helper macro to reduce boilerplate
    macro_rules! active_sel {
        ($gat:expr) => {
            $gat().map(|t| (get_selected_paths(&t.selection_model), t))
        };
    }

    // --- Copy ---
    let clip = clipboard.clone(); let gat = get_active_tab.clone(); let toast = toast_overlay.clone();
    let copy_action = gio::SimpleAction::new("copy", None);
    copy_action.connect_activate(move |_, _| {
        if let Some((paths, _)) = active_sel!(gat) {
            if !paths.is_empty() {
                let c = paths.len();
                *clip.borrow_mut() = Some(ClipboardState { operation: ClipboardOp::Copy, paths });
                toast.add_toast(adw::Toast::new(&format!("{c} item(s) copied")));
            }
        }
    });
    window.add_action(&copy_action);

    // --- Cut ---
    let clip = clipboard.clone(); let gat = get_active_tab.clone(); let toast = toast_overlay.clone();
    let cut_action = gio::SimpleAction::new("cut", None);
    cut_action.connect_activate(move |_, _| {
        if let Some((paths, _)) = active_sel!(gat) {
            if !paths.is_empty() {
                let c = paths.len();
                *clip.borrow_mut() = Some(ClipboardState { operation: ClipboardOp::Cut, paths });
                toast.add_toast(adw::Toast::new(&format!("{c} item(s) cut")));
            }
        }
    });
    window.add_action(&cut_action);

    // --- Paste ---
    let clip = clipboard.clone(); let gat = get_active_tab.clone(); let toast = toast_overlay.clone();
    let bc = breadcrumb.clone(); let bb = header.back_btn.clone(); let fb = header.forward_btn.clone(); let tv = tab_view.clone();
    let paste_action = gio::SimpleAction::new("paste", None);
    paste_action.connect_activate(move |_, _| {
        let state = clip.borrow().clone();
        if let (Some(state), Some(tab)) = (state, gat()) {
            let dest_dir = tab.nav.borrow().current.clone();
            let is_cut = state.operation == ClipboardOp::Cut;
            let load = make_load_for_tab(&tab, &bc, &bb, &fb, &tv);
            for source in &state.paths {
                let name = source.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let dest = dest_dir.join(&name);
                let src = source.clone(); let op = state.operation;
                let load = load.clone(); let toast = toast.clone(); let dd = dest_dir.clone();
                glib::spawn_future_local(async move {
                    let r = runtime::spawn(async move {
                        if op == ClipboardOp::Copy { crate::operations::file_ops::copy_file(&src, &dest).await }
                        else { crate::operations::file_ops::move_file(&src, &dest).await }
                    }).await;
                    match r {
                        Ok(Ok(())) => { toast.add_toast(adw::Toast::new("Paste complete")); (load)(dd); }
                        Ok(Err(e)) => { toast.add_toast(adw::Toast::new(&format!("Paste failed: {e}"))); }
                        Err(e) => { toast.add_toast(adw::Toast::new(&format!("Paste failed: {e}"))); }
                    }
                });
            }
            if is_cut { *clip.borrow_mut() = None; }
        }
    });
    window.add_action(&paste_action);

    // --- Copy Path ---
    let gat = get_active_tab.clone(); let win_ref = window.clone(); let toast = toast_overlay.clone();
    let copy_path_action = gio::SimpleAction::new("copy-path", None);
    copy_path_action.connect_activate(move |_, _| {
        if let Some((paths, _)) = active_sel!(gat) {
            if let Some(first) = paths.first() {
                gtk::prelude::WidgetExt::display(&win_ref).clipboard().set_text(&first.to_string_lossy());
                toast.add_toast(adw::Toast::new("Path copied"));
            }
        }
    });
    window.add_action(&copy_path_action);

    // --- Select All ---
    let gat = get_active_tab.clone();
    let sa = gio::SimpleAction::new("select-all", None);
    sa.connect_activate(move |_, _| { if let Some(t) = gat() { t.selection_model.select_all(); } });
    window.add_action(&sa);

    // --- Invert ---
    let gat = get_active_tab.clone();
    let inv = gio::SimpleAction::new("invert-selection", None);
    inv.connect_activate(move |_, _| {
        if let Some(t) = gat() {
            let n = t.selection_model.n_items();
            for i in 0..n { if t.selection_model.is_selected(i) { t.selection_model.unselect_item(i); } else { t.selection_model.select_item(i, false); } }
        }
    });
    window.add_action(&inv);

    // --- Trash ---
    let gat = get_active_tab.clone(); let toast = toast_overlay.clone(); let win = window.clone();
    let confirm = settings.borrow().confirm_trash;
    let bc = breadcrumb.clone(); let bb = header.back_btn.clone(); let fb = header.forward_btn.clone(); let tv = tab_view.clone();
    let trash_action = gio::SimpleAction::new("trash", None);
    trash_action.connect_activate(move |_, _| {
        if let Some((paths, tab)) = active_sel!(gat) {
            if paths.is_empty() { return; }
            let load = make_load_for_tab(&tab, &bc, &bb, &fb, &tv);
            if confirm {
                let count = paths.len();
                let msg = if count == 1 { let n = paths[0].file_name().unwrap_or_default().to_string_lossy().into_owned(); format!("Move \"{n}\" to trash?") } else { format!("Move {count} items to trash?") };
                let dlg = adw::MessageDialog::new(Some(&win), Some("Move to Trash"), Some(&msg));
                dlg.add_response("cancel", "Cancel"); dlg.add_response("trash", "Move to Trash");
                dlg.set_response_appearance("trash", adw::ResponseAppearance::Destructive);
                let load = load.clone(); let toast = toast.clone(); let nav = tab.nav.clone();
                dlg.connect_response(None, move |d, r| { if r == "trash" { do_trash(&paths, &nav, &load, &toast); } d.close(); });
                dlg.present();
            } else {
                do_trash(&paths, &tab.nav, &load, &toast);
            }
        }
    });
    window.add_action(&trash_action);

    // --- New Folder ---
    let gat = get_active_tab.clone(); let toast = toast_overlay.clone(); let win = window.clone();
    let bc = breadcrumb.clone(); let bb = header.back_btn.clone(); let fb = header.forward_btn.clone(); let tv = tab_view.clone();
    let nf = gio::SimpleAction::new("new-folder", None);
    nf.connect_activate(move |_, _| {
        if let Some(tab) = gat() {
            let dir = tab.nav.borrow().current.clone();
            let load = make_load_for_tab(&tab, &bc, &bb, &fb, &tv);
            let toast = toast.clone();
            let dlg = adw::MessageDialog::new(Some(&win), Some("New Folder"), Some("Enter folder name:"));
            dlg.add_response("cancel", "Cancel"); dlg.add_response("create", "Create");
            dlg.set_response_appearance("create", adw::ResponseAppearance::Suggested);
            let entry = gtk::Entry::new(); entry.set_placeholder_text(Some("Folder name")); dlg.set_extra_child(Some(&entry));
            let er = entry.clone();
            dlg.connect_response(None, move |d, r| {
                if r == "create" { let name = er.text().to_string(); if !name.is_empty() { let p = dir.join(&name); let l = load.clone(); let t = toast.clone(); let dd = dir.clone();
                    glib::spawn_future_local(async move { let r = runtime::spawn(async move { crate::operations::file_ops::create_directory(&p).await }).await;
                        match r { Ok(Ok(())) => (l)(dd), Ok(Err(e)) => { t.add_toast(adw::Toast::new(&format!("Failed: {e}"))); } Err(e) => { t.add_toast(adw::Toast::new(&format!("Failed: {e}"))); } } }); } } d.close(); });
            dlg.present();
        }
    });
    window.add_action(&nf);

    // --- Rename ---
    let gat = get_active_tab.clone(); let toast = toast_overlay.clone(); let win = window.clone();
    let bc = breadcrumb.clone(); let bb = header.back_btn.clone(); let fb = header.forward_btn.clone(); let tv = tab_view.clone();
    let rn = gio::SimpleAction::new("rename", None);
    rn.connect_activate(move |_, _| {
        if let Some((paths, tab)) = active_sel!(gat) {
            if let Some(old) = paths.first() {
                let old_name = old.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let parent = old.parent().unwrap_or(old).to_path_buf(); let old = old.clone();
                let load = make_load_for_tab(&tab, &bc, &bb, &fb, &tv); let toast = toast.clone();
                let dlg = adw::MessageDialog::new(Some(&win), Some("Rename"), Some(&format!("Rename \"{old_name}\":")));
                dlg.add_response("cancel", "Cancel"); dlg.add_response("rename", "Rename");
                dlg.set_response_appearance("rename", adw::ResponseAppearance::Suggested);
                let entry = gtk::Entry::new(); entry.set_text(&old_name); dlg.set_extra_child(Some(&entry));
                let er = entry.clone(); let on = old_name.clone();
                dlg.connect_response(None, move |d, r| {
                    if r == "rename" { let nn = er.text().to_string(); if !nn.is_empty() && nn != on { let np = parent.join(&nn); let f = old.clone(); let l = load.clone(); let t = toast.clone(); let pp = parent.clone();
                        glib::spawn_future_local(async move { let r = runtime::spawn(async move { crate::operations::file_ops::rename(&f, &np).await }).await;
                            match r { Ok(Ok(())) => (l)(pp), Ok(Err(e)) => { t.add_toast(adw::Toast::new(&format!("Failed: {e}"))); } Err(e) => { t.add_toast(adw::Toast::new(&format!("Failed: {e}"))); } } }); } } d.close(); });
                dlg.present();
            }
        }
    });
    window.add_action(&rn);

    // --- Open Terminal ---
    let gat = get_active_tab.clone();
    let ot = gio::SimpleAction::new("open-terminal", None);
    ot.connect_activate(move |_, _| {
        if let Some(tab) = gat() {
            let dir = tab.nav.borrow().current.clone();
            let term_env = std::env::var("TERMINAL").ok();
            let terminals = ["kgx", "gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "xterm"];
            let candidates: Vec<&str> = term_env.as_deref().into_iter().chain(terminals.iter().copied()).collect();
            for t in candidates { if std::process::Command::new(t).current_dir(&dir).spawn().is_ok() { return; } }
        }
    });
    window.add_action(&ot);

    // --- Open With ---
    let gat = get_active_tab.clone(); let win = window.clone();
    let ow = gio::SimpleAction::new("open-with", None);
    ow.connect_activate(move |_, _| {
        if let Some((paths, _)) = active_sel!(gat) {
            if let Some(path) = paths.first() {
                let file = gio::File::for_path(path);
                let launcher = gtk::FileLauncher::new(Some(&file));
                launcher.set_always_ask(true);
                launcher.launch(Some(&win), gio::Cancellable::NONE, |r| { if let Err(e) = r { log::warn!("Open With failed: {e}"); } });
            }
        }
    });
    window.add_action(&ow);

    // --- Properties ---
    let gat = get_active_tab.clone(); let win = window.clone();
    let pr = gio::SimpleAction::new("properties", None);
    pr.connect_activate(move |_, _| {
        if let Some((paths, _)) = active_sel!(gat) { if let Some(p) = paths.first() { crate::widgets::properties_dialog::show_properties(&win, p); } }
    });
    window.add_action(&pr);

    // --- Add Bookmark ---
    let gat = get_active_tab.clone(); let sb = sidebar.clone(); let toast = toast_overlay.clone();
    let ab = gio::SimpleAction::new("add-bookmark", None);
    ab.connect_activate(move |_, _| {
        if let Some(tab) = gat() {
            let dir = tab.nav.borrow().current.clone();
            sb.add_bookmark(dir.clone());
            let name = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
            toast.add_toast(adw::Toast::new(&format!("Bookmarked \"{name}\"")));
        }
    });
    window.add_action(&ab);

    // --- Open in New Tab ---
    let gat = get_active_tab.clone(); let ct = create_tab.clone();
    let oint = gio::SimpleAction::new("open-in-new-tab", None);
    oint.connect_activate(move |_, _| {
        if let Some((paths, _tab)) = active_sel!(gat) {
            if let Some(p) = paths.first() {
                if std::path::Path::new(p).is_dir() { ct(p.clone()); }
            }
        } else if let Some(tab) = gat() {
            ct(tab.current_path());
        }
    });
    window.add_action(&oint);

    // --- Toggle hidden ---
    let hidden = show_hidden.clone(); let toggle_btn = header.hidden_toggle.clone();
    let settings_h = settings.clone(); let tabs_h = tabs.clone();
    let th = gio::SimpleAction::new("toggle-hidden", None);
    th.connect_activate(move |_, _| {
        let v = !hidden.get(); hidden.set(v); toggle_btn.set_active(v);
        for t in tabs_h.borrow().iter() { t.filter.changed(gtk::FilterChange::Different); }
        settings_h.borrow_mut().show_hidden_files = v; settings_h.borrow().save();
    });
    window.add_action(&th);

    // --- Location bar ---
    let bc = breadcrumb.clone();
    let lb = gio::SimpleAction::new("location-bar", None);
    lb.connect_activate(move |_, _| { bc.enter_edit_mode(); });
    window.add_action(&lb);

    // --- Filter toggle ---
    let gat = get_active_tab.clone(); let search_btn = header.search_btn.clone();
    let ft = gio::SimpleAction::new("toggle-filter", None);
    ft.connect_activate(move |_, _| {
        if let Some(tab) = gat() { tab.filter_bar.toggle(); search_btn.set_active(tab.filter_bar.is_visible()); }
    });
    window.add_action(&ft);

    // --- Nav actions ---
    add_nav_action_tab(window, "go-back", get_active_tab, breadcrumb, &header.back_btn, &header.forward_btn, tab_view, |n| { if n.go_back() { Some(n.current.clone()) } else { None } });
    add_nav_action_tab(window, "go-forward", get_active_tab, breadcrumb, &header.back_btn, &header.forward_btn, tab_view, |n| { if n.go_forward() { Some(n.current.clone()) } else { None } });
    add_nav_action_tab(window, "go-up", get_active_tab, breadcrumb, &header.back_btn, &header.forward_btn, tab_view, |n| { if n.go_up() { Some(n.current.clone()) } else { None } });

    // --- Zoom actions ---
    let is = icon_size.clone(); let s = settings.clone();
    let zi = gio::SimpleAction::new("zoom-in", None);
    zi.connect_activate(move |_, _| { let n = (is.get()+8).min(128); is.set(n); s.borrow_mut().icon_size=n; s.borrow().save(); });
    window.add_action(&zi);
    let is = icon_size.clone(); let s = settings.clone();
    let zo = gio::SimpleAction::new("zoom-out", None);
    zo.connect_activate(move |_, _| { let n = (is.get()-8).max(24); is.set(n); s.borrow_mut().icon_size=n; s.borrow().save(); });
    window.add_action(&zo);
    let is = icon_size.clone(); let s = settings.clone();
    let zr = gio::SimpleAction::new("zoom-reset", None);
    zr.connect_activate(move |_, _| { is.set(48); s.borrow_mut().icon_size=48; s.borrow().save(); });
    window.add_action(&zr);

    // --- New Tab ---
    let ct = create_tab.clone(); let gat = get_active_tab.clone();
    let nt = gio::SimpleAction::new("new-tab", None);
    nt.connect_activate(move |_, _| {
        let path = gat().map(|t| t.current_path()).unwrap_or_else(glib::home_dir);
        ct(path);
    });
    window.add_action(&nt);

    // --- Close Tab ---
    let tv = tab_view.clone();
    let ct_close = gio::SimpleAction::new("close-tab", None);
    ct_close.connect_activate(move |_, _| {
        if let Some(page) = tv.selected_page() {
            if tv.n_pages() > 1 { tv.close_page(&page); }
        }
    });
    window.add_action(&ct_close);

    // --- Preferences ---
    let win = window.clone(); let s = settings.clone();
    let pref = gio::SimpleAction::new("preferences", None);
    pref.connect_activate(move |_, _| { crate::widgets::preferences_window::show_preferences(&win, s.clone()); });
    window.add_action(&pref);

    // --- Toggle Terminal (F4) ---
    let term = terminal.clone();
    let gat2 = get_active_tab.clone();
    let toggle_term = gio::SimpleAction::new("toggle-terminal", None);
    toggle_term.connect_activate(move |_, _| {
        term.toggle();
        // Sync terminal to current directory
        if term.is_visible() {
            if let Some(tab) = gat2() {
                term.cd(&tab.current_path());
            }
        }
    });
    window.add_action(&toggle_term);

    // --- Toggle Dual Pane (F3) ---
    let dp = dual_pane.clone();
    let gat3 = get_active_tab.clone();
    let toggle_dual = gio::SimpleAction::new("toggle-dual-pane", None);
    toggle_dual.connect_activate(move |_, _| {
        dp.toggle();
        if dp.is_visible() {
            if let Some(tab) = gat3() {
                let path = tab.current_path();
                dp.left.load_directory(path.clone());
                dp.right.load_directory(path);
            }
        }
    });
    window.add_action(&toggle_dual);

    // --- Toggle Miller Columns (F5) ---
    let mr = miller_revealer.clone();
    let ml = miller.clone();
    let gat4 = get_active_tab.clone();
    let toggle_miller = gio::SimpleAction::new("toggle-miller", None);
    toggle_miller.connect_activate(move |_, _| {
        let visible = mr.reveals_child();
        mr.set_reveal_child(!visible);
        if !visible {
            if let Some(tab) = gat4() {
                ml.navigate_to(tab.current_path());
            }
        }
    });
    window.add_action(&toggle_miller);

    // --- Vim-mode navigation helpers ---
    // select-next / select-prev for j/k navigation
    let gat5 = get_active_tab.clone();
    let sel_next = gio::SimpleAction::new("select-next", None);
    sel_next.connect_activate(move |_, _| {
        if let Some(tab) = gat5() {
            let sel = &tab.selection_model;
            let n = sel.n_items();
            if n == 0 { return; }
            // Find first selected, move to next
            let mut current = 0;
            for i in 0..n { if sel.is_selected(i) { current = i; break; } }
            let next = (current + 1).min(n - 1);
            sel.select_item(next, true);
        }
    });
    window.add_action(&sel_next);

    let gat6 = get_active_tab.clone();
    let sel_prev = gio::SimpleAction::new("select-prev", None);
    sel_prev.connect_activate(move |_, _| {
        if let Some(tab) = gat6() {
            let sel = &tab.selection_model;
            let n = sel.n_items();
            if n == 0 { return; }
            let mut current = 0;
            for i in 0..n { if sel.is_selected(i) { current = i; break; } }
            let prev = if current > 0 { current - 1 } else { 0 };
            sel.select_item(prev, true);
        }
    });
    window.add_action(&sel_prev);

    // go-first / go-last
    let gat7 = get_active_tab.clone();
    let go_first = gio::SimpleAction::new("go-first", None);
    go_first.connect_activate(move |_, _| {
        if let Some(tab) = gat7() {
            if tab.selection_model.n_items() > 0 { tab.selection_model.select_item(0, true); }
        }
    });
    window.add_action(&go_first);

    let gat8 = get_active_tab.clone();
    let go_last = gio::SimpleAction::new("go-last", None);
    go_last.connect_activate(move |_, _| {
        if let Some(tab) = gat8() {
            let n = tab.selection_model.n_items();
            if n > 0 { tab.selection_model.select_item(n - 1, true); }
        }
    });
    window.add_action(&go_last);

    // activate-item (l in vim mode / Enter)
    let gat9 = get_active_tab.clone();
    let bc_act = breadcrumb.clone(); let bb_act = header.back_btn.clone(); let fb_act = header.forward_btn.clone(); let tv_act = tab_view.clone();
    let activate_item = gio::SimpleAction::new("activate-item", None);
    activate_item.connect_activate(move |_, _| {
        if let Some(tab) = gat9() {
            let sel = &tab.selection_model;
            for i in 0..sel.n_items() {
                if sel.is_selected(i) {
                    if let Some(item) = sel.item(i) {
                        if let Some(entry) = item.downcast_ref::<FileEntry>() {
                            let path = PathBuf::from(entry.path());
                            if entry.is_dir() {
                                tab.nav.borrow_mut().navigate_to(path.clone());
                                let load = make_load_for_tab(&tab, &bc_act, &bb_act, &fb_act, &tv_act);
                                (load)(path);
                            } else {
                                open_file(&path);
                            }
                        }
                    }
                    break;
                }
            }
        }
    });
    window.add_action(&activate_item);

    // --- Shortcuts dialog ---
    let win = window.clone();
    let sk = gio::SimpleAction::new("show-shortcuts", None);
    sk.connect_activate(move |_, _| { show_shortcuts_window(&win); });
    window.add_action(&sk);

    // --- User Guide ---
    let win = window.clone();
    let guide_action = gio::SimpleAction::new("show-guide", None);
    guide_action.connect_activate(move |_, _| {
        crate::widgets::guide_window::show_guide(&win);
    });
    window.add_action(&guide_action);

    // --- About ---
    let win = window.clone();
    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| {
        crate::widgets::about_dialog::show_about(&win);
    });
    window.add_action(&about_action);

    // --- Keyboard shortcuts ---
    let sc = gtk::ShortcutController::new();
    sc.set_scope(gtk::ShortcutScope::Managed);
    let mut shortcuts: Vec<(&str, &str)> = vec![
        ("<Control>c", "win.copy"), ("<Control>x", "win.cut"), ("<Control>v", "win.paste"),
        ("<Control><Shift>c", "win.copy-path"), ("<Control>a", "win.select-all"), ("<Control>i", "win.invert-selection"),
        ("Delete", "win.trash"), ("F2", "win.rename"),
        ("<Control>h", "win.toggle-hidden"), ("<Control>l", "win.location-bar"), ("<Control>f", "win.toggle-filter"),
        ("<Alt>Left", "win.go-back"), ("<Alt>Right", "win.go-forward"), ("<Alt>Up", "win.go-up"), ("BackSpace", "win.go-back"),
        ("<Control>plus", "win.zoom-in"), ("<Control>equal", "win.zoom-in"), ("<Control>minus", "win.zoom-out"), ("<Control>0", "win.zoom-reset"),
        ("<Alt>Return", "win.properties"), ("<Control><Alt>t", "win.open-terminal"), ("<Control>comma", "win.preferences"),
        ("<Control>d", "win.add-bookmark"),
        ("<Control>t", "win.new-tab"), ("<Control>w", "win.close-tab"),
        // Phase 2
        ("F4", "win.toggle-terminal"),
        ("F3", "win.toggle-dual-pane"),
        ("F5", "win.toggle-miller"),
        ("F1", "win.show-guide"),
        ("Return", "win.activate-item"),
    ];

    // Add vim keybindings if enabled
    let kb_config = KeybindingConfig::load();
    if kb_config.vim_mode {
        for (key, action) in KeybindingConfig::vim_shortcuts() {
            shortcuts.push((key, action));
        }
    }

    for (key, action) in &shortcuts {
        sc.add_shortcut(gtk::Shortcut::new(gtk::ShortcutTrigger::parse_string(key), Some(gtk::NamedAction::new(action))));
    }
    window.add_controller(sc);
}

fn add_nav_action_tab<F>(
    window: &adw::ApplicationWindow,
    name: &str,
    get_active_tab: &ActiveTabFn,
    breadcrumb: &Rc<BreadcrumbBar>,
    back_btn: &gtk::Button,
    fwd_btn: &gtk::Button,
    tab_view: &adw::TabView,
    action: F,
) where F: Fn(&mut NavigationState) -> Option<PathBuf> + 'static {
    let gat = get_active_tab.clone(); let bc = breadcrumb.clone(); let bb = back_btn.clone(); let fb = fwd_btn.clone(); let tv = tab_view.clone();
    let a = gio::SimpleAction::new(name, None);
    a.connect_activate(move |_, _| {
        if let Some(tab) = gat() {
            let path = action(&mut tab.nav.borrow_mut());
            if let Some(p) = path { let load = make_load_for_tab(&tab, &bc, &bb, &fb, &tv); (load)(p); }
        }
    });
    window.add_action(&a);
}

fn make_load_for_tab(
    tab: &Rc<TabState>,
    breadcrumb: &Rc<BreadcrumbBar>,
    back_btn: &gtk::Button,
    fwd_btn: &gtk::Button,
    tab_view: &adw::TabView,
) -> Rc<dyn Fn(PathBuf)> {
    let tab = tab.clone(); let breadcrumb = breadcrumb.clone();
    let back_btn = back_btn.clone(); let fwd_btn = fwd_btn.clone(); let tab_view = tab_view.clone();

    Rc::new(move |path: PathBuf| {
        tab.content.show_loading();
        let tab = tab.clone(); let breadcrumb = breadcrumb.clone();
        let back_btn = back_btn.clone(); let fwd_btn = fwd_btn.clone(); let tab_view = tab_view.clone();

        breadcrumb.set_path(&path, {
            let tab = tab.clone(); let back_btn = back_btn.clone(); let fwd_btn = fwd_btn.clone();
            move |target| {
                { let mut n = tab.nav.borrow_mut(); n.navigate_to(target.clone()); back_btn.set_sensitive(n.can_go_back()); fwd_btn.set_sensitive(n.can_go_forward()); }
                tab.content.show_loading();
                load_path_async(target, tab.file_model.clone(), tab.status_bar.clone(), tab.monitor.clone(), tab.content.clone(), tab.filter.clone(), tab.filter_model.clone());
            }
        });

        { let n = tab.nav.borrow(); back_btn.set_sensitive(n.can_go_back()); fwd_btn.set_sensitive(n.can_go_forward()); }

        let title = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "/".to_string());
        tab_view.page(&tab.container).set_title(&title);

        load_path_async(path, tab.file_model.clone(), tab.status_bar.clone(), tab.monitor.clone(), tab.content.clone(), tab.filter.clone(), tab.filter_model.clone());
    })
}

fn do_trash(paths: &[PathBuf], nav: &Rc<RefCell<NavigationState>>, load: &Rc<dyn Fn(PathBuf)>, toast_overlay: &adw::ToastOverlay) {
    let dest_dir = nav.borrow().current.clone();
    let mut trashed = Vec::new();
    for path in paths {
        match crate::operations::trash_ops::move_to_trash(path) {
            Ok(()) => trashed.push(path.clone()),
            Err(e) => { toast_overlay.add_toast(adw::Toast::new(&format!("Trash failed: {e}"))); }
        }
    }
    if !trashed.is_empty() {
        let c = trashed.len();
        let toast = adw::Toast::new(&format!("Moved {c} item{} to Trash", if c==1 {""} else {"s"}));
        toast.set_button_label(Some("Undo")); toast.set_timeout(5);
        let load_undo = load.clone(); let dd = dest_dir.clone();
        toast.connect_button_clicked(move |_| {
            for path in &trashed {
                if let Some(name) = path.file_name() {
                    let tf = dirs::data_dir().unwrap_or_default().join("Trash/files").join(name);
                    if tf.exists() { let _ = std::fs::rename(&tf, path); let ip = dirs::data_dir().unwrap_or_default().join("Trash/info").join(format!("{}.trashinfo", name.to_string_lossy())); let _ = std::fs::remove_file(ip); }
                }
            }
            (load_undo)(dd.clone());
        });
        toast_overlay.add_toast(toast);
    }
    (load)(dest_dir);
}

fn show_shortcuts_window(parent: &adw::ApplicationWindow) {
    let dlg = adw::MessageDialog::new(Some(parent), Some("Keyboard Shortcuts"), None);
    dlg.add_response("close", "Close");
    let grid = gtk::Grid::new(); grid.set_column_spacing(24); grid.set_row_spacing(6); grid.set_margin_start(12); grid.set_margin_end(12);
    let shortcuts = [
        ("Ctrl+C", "Copy"), ("Ctrl+X", "Cut"), ("Ctrl+V", "Paste"), ("Ctrl+Shift+C", "Copy Path"),
        ("Ctrl+A", "Select All"), ("Ctrl+I", "Invert Selection"), ("Delete", "Move to Trash"), ("F2", "Rename"),
        ("Ctrl+H", "Toggle Hidden"), ("Ctrl+L", "Edit Location"), ("Ctrl+F", "Filter"),
        ("Alt+Left", "Back"), ("Alt+Right", "Forward"), ("Alt+Up", "Parent"),
        ("Ctrl++", "Zoom In"), ("Ctrl+-", "Zoom Out"), ("Ctrl+0", "Reset Zoom"),
        ("Alt+Enter", "Properties"), ("Ctrl+Alt+T", "Open Terminal"), ("Ctrl+,", "Preferences"),
        ("Ctrl+D", "Bookmark Folder"), ("Ctrl+T", "New Tab"), ("Ctrl+W", "Close Tab"),
    ];
    for (i, (k, d)) in shortcuts.iter().enumerate() {
        let kl = gtk::Label::new(Some(k)); kl.set_halign(gtk::Align::Start); kl.add_css_class("dim-label"); grid.attach(&kl, 0, i as i32, 1, 1);
        let dl = gtk::Label::new(Some(d)); dl.set_halign(gtk::Align::Start); grid.attach(&dl, 1, i as i32, 1, 1);
    }
    dlg.set_extra_child(Some(&grid)); dlg.present();
}

fn get_selected_paths(selection: &gtk::MultiSelection) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for i in 0..selection.n_items() {
        if selection.is_selected(i) { if let Some(item) = selection.item(i) { if let Some(entry) = item.downcast_ref::<FileEntry>() { paths.push(PathBuf::from(entry.path())); } } }
    }
    paths
}

impl Clone for ClipboardState {
    fn clone(&self) -> Self { Self { operation: self.operation, paths: self.paths.clone() } }
}
