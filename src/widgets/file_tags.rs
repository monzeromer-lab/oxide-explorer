use adw::prelude::*;
use std::path::Path;

const TAG_XATTR: &str = "user.oxide.tags";
const COLOR_XATTR: &str = "user.oxide.color";

/// Available color dots
pub const COLORS: &[(&str, &str)] = &[
    ("red", "#e74c3c"),
    ("orange", "#e67e22"),
    ("yellow", "#f1c40f"),
    ("green", "#2ecc71"),
    ("blue", "#3498db"),
    ("purple", "#9b59b6"),
    ("gray", "#95a5a6"),
];

/// Get tags for a file from xattrs
pub fn get_tags(path: &Path) -> Vec<String> {
    match xattr::get(path, TAG_XATTR) {
        Ok(Some(data)) => {
            String::from_utf8_lossy(&data)
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Set tags for a file via xattrs
pub fn set_tags(path: &Path, tags: &[String]) {
    let value = tags.join(",");
    let _ = xattr::set(path, TAG_XATTR, value.as_bytes());
}

/// Get color dot for a file
pub fn get_color(path: &Path) -> Option<String> {
    match xattr::get(path, COLOR_XATTR) {
        Ok(Some(data)) => {
            let color = String::from_utf8_lossy(&data).to_string();
            if color.is_empty() { None } else { Some(color) }
        }
        _ => None,
    }
}

/// Set color dot for a file
pub fn set_color(path: &Path, color: &str) {
    let _ = xattr::set(path, COLOR_XATTR, color.as_bytes());
}

/// Remove color dot
pub fn clear_color(path: &Path) {
    let _ = xattr::remove(path, COLOR_XATTR);
}

/// Show tag editing dialog for a file
pub fn show_tag_dialog(parent: &impl IsA<gtk::Window>, path: &Path) {
    let path = path.to_path_buf();

    let dialog = adw::Window::builder()
        .title("Tags & Colors")
        .default_width(350)
        .default_height(400)
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

    let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let title = gtk::Label::new(Some(&name));
    title.add_css_class("title-3");
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    content.append(&title);

    // --- Color dots ---
    let color_group = adw::PreferencesGroup::builder().title("Color").build();
    let color_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    color_box.set_halign(gtk::Align::Center);
    color_box.set_margin_top(4);
    color_box.set_margin_bottom(4);

    let current_color = get_color(&path);
    let path_for_color = path.clone();

    // No color button
    let none_btn = gtk::Button::with_label("None");
    none_btn.add_css_class("flat");
    let p = path_for_color.clone();
    none_btn.connect_clicked(move |_| { clear_color(&p); });
    color_box.append(&none_btn);

    for (name, hex) in COLORS {
        let btn = gtk::Button::new();
        btn.set_width_request(28);
        btn.set_height_request(28);
        // Use inline CSS for the color
        // Color the button via CSS class + inline provider
        let css_provider = gtk::CssProvider::new();
        let class_name = format!("color-dot-{name}");
        css_provider.load_from_string(&format!(
            ".{class_name} {{ background-color: {hex}; border-radius: 50%; min-width: 24px; min-height: 24px; }}"
        ));
        gtk::style_context_add_provider_for_display(
            &btn.display(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        btn.add_css_class(&class_name);
        btn.set_tooltip_text(Some(name));

        let color_name = name.to_string();
        let p = path_for_color.clone();
        btn.connect_clicked(move |_| { set_color(&p, &color_name); });

        // Highlight current color
        if current_color.as_deref() == Some(name) {
            btn.add_css_class("suggested-action");
        }

        color_box.append(&btn);
    }

    let color_row = gtk::ListBoxRow::new();
    color_row.set_child(Some(&color_box));
    color_row.set_activatable(false);
    color_group.add(&color_row);
    content.append(&color_group);

    // --- Tags ---
    let tags_group = adw::PreferencesGroup::builder().title("Tags").build();

    let current_tags = get_tags(&path);
    let tags_label = gtk::Label::new(Some(&current_tags.join(", ")));
    tags_label.set_halign(gtk::Align::Start);
    tags_label.add_css_class("dim-label");
    if current_tags.is_empty() { tags_label.set_text("No tags"); }
    let tags_info_row = adw::ActionRow::builder().title("Current tags").build();
    tags_info_row.add_suffix(&tags_label);
    tags_group.add(&tags_info_row);

    let new_tag_row = adw::EntryRow::builder().title("Add tag").build();
    tags_group.add(&new_tag_row);

    let add_btn = gtk::Button::with_label("Add Tag");
    add_btn.add_css_class("suggested-action");
    add_btn.set_halign(gtk::Align::End);

    let entry = new_tag_row.clone();
    let p = path.clone();
    let tl = tags_label.clone();
    add_btn.connect_clicked(move |_| {
        let tag = entry.text().to_string().trim().to_string();
        if !tag.is_empty() {
            let mut tags = get_tags(&p);
            if !tags.contains(&tag) {
                tags.push(tag);
                set_tags(&p, &tags);
                tl.set_text(&tags.join(", "));
            }
            entry.set_text("");
        }
    });

    content.append(&tags_group);
    content.append(&add_btn);

    toolbar_view.set_content(Some(&content));
    dialog.set_content(Some(&toolbar_view));
    dialog.present();
}
