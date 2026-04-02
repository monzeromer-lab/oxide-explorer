use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::state::settings::{Settings, SortColumn, ViewMode};

pub fn show_preferences(parent: &impl IsA<gtk::Window>, settings: Rc<RefCell<Settings>>) {
    let window = adw::PreferencesWindow::builder()
        .title("Preferences")
        .modal(true)
        .transient_for(parent)
        .default_width(500)
        .default_height(450)
        .build();

    let page = adw::PreferencesPage::new();

    // --- Appearance group ---
    let appearance_group = adw::PreferencesGroup::builder()
        .title("Appearance")
        .build();

    // Default view mode
    let view_row = adw::ComboRow::builder()
        .title("Default View")
        .subtitle("View mode when opening new directories")
        .build();
    let view_model = gtk::StringList::new(&["Icon View", "Details View"]);
    view_row.set_model(Some(&view_model));
    let current_view = match settings.borrow().default_view {
        ViewMode::Icon => 0,
        ViewMode::Details => 1,
    };
    view_row.set_selected(current_view);
    let s = settings.clone();
    view_row.connect_selected_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.default_view = match row.selected() {
            0 => ViewMode::Icon,
            _ => ViewMode::Details,
        };
        settings.save();
    });
    appearance_group.add(&view_row);

    // Icon size
    let icon_row = adw::SpinRow::builder()
        .title("Icon Size")
        .subtitle("Size of icons in grid view")
        .adjustment(&gtk::Adjustment::new(
            settings.borrow().icon_size as f64,
            24.0,
            128.0,
            8.0,
            16.0,
            0.0,
        ))
        .build();
    let s = settings.clone();
    icon_row.connect_value_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.icon_size = row.value() as i32;
        settings.save();
    });
    appearance_group.add(&icon_row);

    page.add(&appearance_group);

    // --- Behavior group ---
    let behavior_group = adw::PreferencesGroup::builder()
        .title("Behavior")
        .build();

    // Show hidden files
    let hidden_row = adw::SwitchRow::builder()
        .title("Show Hidden Files")
        .subtitle("Show files starting with a dot")
        .active(settings.borrow().show_hidden_files)
        .build();
    let s = settings.clone();
    hidden_row.connect_active_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.show_hidden_files = row.is_active();
        settings.save();
    });
    behavior_group.add(&hidden_row);

    // Confirm trash
    let trash_row = adw::SwitchRow::builder()
        .title("Confirm Before Trash")
        .subtitle("Show a confirmation dialog before moving items to trash")
        .active(settings.borrow().confirm_trash)
        .build();
    let s = settings.clone();
    trash_row.connect_active_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.confirm_trash = row.is_active();
        settings.save();
    });
    behavior_group.add(&trash_row);

    page.add(&behavior_group);

    // --- Sorting group ---
    let sort_group = adw::PreferencesGroup::builder()
        .title("Sorting")
        .build();

    let sort_row = adw::ComboRow::builder()
        .title("Sort By")
        .build();
    let sort_model = gtk::StringList::new(&["Name", "Size", "Date"]);
    sort_row.set_model(Some(&sort_model));
    let current_sort = match settings.borrow().sort_by {
        SortColumn::Name => 0,
        SortColumn::Size => 1,
        SortColumn::Date => 2,
    };
    sort_row.set_selected(current_sort);
    let s = settings.clone();
    sort_row.connect_selected_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.sort_by = match row.selected() {
            0 => SortColumn::Name,
            1 => SortColumn::Size,
            _ => SortColumn::Date,
        };
        settings.save();
    });
    sort_group.add(&sort_row);

    let ascending_row = adw::SwitchRow::builder()
        .title("Ascending Order")
        .active(settings.borrow().sort_ascending)
        .build();
    let s = settings.clone();
    ascending_row.connect_active_notify(move |row| {
        let mut settings = s.borrow_mut();
        settings.sort_ascending = row.is_active();
        settings.save();
    });
    sort_group.add(&ascending_row);

    page.add(&sort_group);

    // --- Keybindings group ---
    let kb_group = adw::PreferencesGroup::builder()
        .title("Keybindings")
        .build();

    let vim_row = adw::SwitchRow::builder()
        .title("Vim-style Keybindings")
        .subtitle("Use h/j/k/l for navigation, d/y/p for file ops (requires restart)")
        .active(crate::state::keybindings::KeybindingConfig::load().vim_mode)
        .build();
    vim_row.connect_active_notify(move |row| {
        let mut kb = crate::state::keybindings::KeybindingConfig::load();
        kb.vim_mode = row.is_active();
        kb.save();
    });
    kb_group.add(&vim_row);

    page.add(&kb_group);

    window.add(&page);
    window.present();
}
