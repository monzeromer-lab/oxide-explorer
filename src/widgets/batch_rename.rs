use adw::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

use crate::utils::runtime;

pub fn show_batch_rename<F>(parent: &impl IsA<gtk::Window>, files: Vec<PathBuf>, on_done: F)
where
    F: Fn() + Clone + 'static,
{
    if files.is_empty() { return; }

    let dialog = adw::Window::builder()
        .title("Batch Rename")
        .default_width(550)
        .default_height(500)
        .modal(true)
        .transient_for(parent)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    // Mode selector
    let mode_group = adw::PreferencesGroup::builder().title("Rename Mode").build();
    let mode_row = adw::ComboRow::builder().title("Mode").build();
    let modes = gtk::StringList::new(&["Find & Replace", "Add Prefix/Suffix", "Numbering", "Change Extension"]);
    mode_row.set_model(Some(&modes));
    mode_group.add(&mode_row);
    content.append(&mode_group);

    // Parameters
    let params_group = adw::PreferencesGroup::builder().title("Parameters").build();
    let find_row = adw::EntryRow::builder().title("Find").build();
    let replace_row = adw::EntryRow::builder().title("Replace with").build();
    let prefix_row = adw::EntryRow::builder().title("Prefix").build();
    let suffix_row = adw::EntryRow::builder().title("Suffix").build();
    let start_row = adw::SpinRow::builder()
        .title("Start number")
        .adjustment(&gtk::Adjustment::new(1.0, 0.0, 99999.0, 1.0, 10.0, 0.0))
        .build();
    let ext_row = adw::EntryRow::builder().title("New extension").build();
    let regex_check = gtk::CheckButton::with_label("Use regex");

    params_group.add(&find_row);
    params_group.add(&replace_row);
    params_group.add(&prefix_row);
    params_group.add(&suffix_row);
    params_group.add(&start_row);
    params_group.add(&ext_row);
    params_group.add(&regex_check);
    content.append(&params_group);

    // Preview
    let preview_group = adw::PreferencesGroup::builder().title("Preview").build();
    let preview_rows: Rc<Vec<adw::ActionRow>> = Rc::new(
        files.iter().take(20).map(|f| {
            let name = f.file_name().unwrap_or_default().to_string_lossy().into_owned();
            adw::ActionRow::builder()
                .title(&name)
                .subtitle(&name)
                .build()
        }).collect()
    );
    for row in preview_rows.iter() {
        preview_group.add(row);
    }
    let preview_scroll = gtk::ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_group));
    preview_scroll.set_vexpand(true);
    content.append(&preview_scroll);

    // Update preview on parameter change
    let update_preview = {
        let files = files.clone();
        let preview_rows = preview_rows.clone();
        let find = find_row.clone();
        let replace = replace_row.clone();
        let prefix = prefix_row.clone();
        let suffix = suffix_row.clone();
        let start = start_row.clone();
        let ext = ext_row.clone();
        let mode = mode_row.clone();
        let regex = regex_check.clone();
        Rc::new(move || {
            let m = mode.selected();
            for (i, row) in preview_rows.iter().enumerate() {
                if i >= files.len() { break; }
                let name = files[i].file_name().unwrap_or_default().to_string_lossy().into_owned();
                let new_name = compute_new_name(&name, m, &find.text(), &replace.text(), &prefix.text(), &suffix.text(), start.value() as u32 + i as u32, &ext.text(), regex.is_active());
                row.set_subtitle(&new_name);
            }
        })
    };

    // Wire change signals
    let up = update_preview.clone();
    find_row.connect_changed(move |_| up());
    let up = update_preview.clone();
    replace_row.connect_changed(move |_| up());
    let up = update_preview.clone();
    prefix_row.connect_changed(move |_| up());
    let up = update_preview.clone();
    suffix_row.connect_changed(move |_| up());
    let up = update_preview.clone();
    ext_row.connect_changed(move |_| up());
    let up = update_preview.clone();
    mode_row.connect_selected_notify(move |_| up());
    let up = update_preview.clone();
    start_row.connect_value_notify(move |_| up());

    // Rename button
    let rename_btn = gtk::Button::with_label("Rename All");
    rename_btn.add_css_class("suggested-action");
    rename_btn.set_halign(gtk::Align::End);

    let dlg = dialog.clone();
    let find = find_row.clone();
    let replace = replace_row.clone();
    let prefix = prefix_row.clone();
    let suffix = suffix_row.clone();
    let start = start_row.clone();
    let ext = ext_row.clone();
    let mode = mode_row.clone();
    let regex = regex_check.clone();
    rename_btn.connect_clicked(move |_| {
        let m = mode.selected();
        let renames: Vec<(PathBuf, PathBuf)> = files.iter().enumerate().map(|(i, f)| {
            let name = f.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let new_name = compute_new_name(&name, m, &find.text(), &replace.text(), &prefix.text(), &suffix.text(), start.value() as u32 + i as u32, &ext.text(), regex.is_active());
            let new_path = f.parent().unwrap_or(f).join(&new_name);
            (f.clone(), new_path)
        }).collect();

        let on_done = on_done.clone();
        glib::spawn_future_local(async move {
            let _ = runtime::spawn(async move {
                for (from, to) in &renames {
                    if from != to {
                        let _ = tokio::fs::rename(from, to).await;
                    }
                }
            }).await;
            on_done();
        });
        dlg.close();
    });
    content.append(&rename_btn);

    toolbar_view.set_content(Some(&content));
    dialog.set_content(Some(&toolbar_view));
    dialog.present();

    // Trigger initial preview
    (update_preview)();
}

fn compute_new_name(
    name: &str, mode: u32,
    find: &str, replace: &str,
    prefix: &str, suffix: &str,
    number: u32, new_ext: &str,
    use_regex: bool,
) -> String {
    match mode {
        0 => { // Find & Replace
            if find.is_empty() { return name.to_string(); }
            if use_regex {
                regex::Regex::new(find)
                    .map(|re| re.replace_all(name, replace).into_owned())
                    .unwrap_or_else(|_| name.to_string())
            } else {
                name.replace(find, replace)
            }
        }
        1 => { // Prefix/Suffix
            format!("{prefix}{name}{suffix}")
        }
        2 => { // Numbering
            let stem = std::path::Path::new(name).file_stem().unwrap_or_default().to_string_lossy();
            let ext = std::path::Path::new(name).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
            if !prefix.is_empty() {
                format!("{prefix}{number:03}{ext}")
            } else {
                format!("{stem}_{number:03}{ext}")
            }
        }
        3 => { // Change Extension
            let stem = std::path::Path::new(name).file_stem().unwrap_or_default().to_string_lossy();
            if new_ext.is_empty() { stem.into_owned() }
            else if new_ext.starts_with('.') { format!("{stem}{new_ext}") }
            else { format!("{stem}.{new_ext}") }
        }
        _ => name.to_string(),
    }
}
