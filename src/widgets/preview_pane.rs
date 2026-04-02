use adw::prelude::*;
use std::path::Path;

pub struct PreviewPane {
    pub widget: gtk::Revealer,
    #[allow(dead_code)]
    content_box: gtk::Box,
    title_label: gtk::Label,
    image: gtk::Picture,
    text_view: gtk::TextView,
    text_scroll: gtk::ScrolledWindow,
    info_label: gtk::Label,
}

impl PreviewPane {
    pub fn new() -> Self {
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        content_box.set_width_request(300);
        content_box.set_margin_start(8);
        content_box.set_margin_end(8);
        content_box.set_margin_top(8);

        // Title
        let title_label = gtk::Label::new(None);
        title_label.set_halign(gtk::Align::Start);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title_label.add_css_class("heading");
        content_box.append(&title_label);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        content_box.append(&sep);

        // Image preview
        let image = gtk::Picture::new();
        image.set_can_shrink(true);
        image.set_content_fit(gtk::ContentFit::Contain);
        image.set_visible(false);
        image.set_height_request(200);
        content_box.append(&image);

        // Text preview
        let text_view = gtk::TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_wrap_mode(gtk::WrapMode::WordChar);
        text_view.set_monospace(true);
        text_view.add_css_class("dim-label");

        let text_scroll = gtk::ScrolledWindow::new();
        text_scroll.set_child(Some(&text_view));
        text_scroll.set_vexpand(true);
        text_scroll.set_visible(false);
        content_box.append(&text_scroll);

        // Info label (for non-previewable files)
        let info_label = gtk::Label::new(None);
        info_label.set_halign(gtk::Align::Start);
        info_label.set_valign(gtk::Align::Start);
        info_label.set_wrap(true);
        info_label.add_css_class("dim-label");
        info_label.set_visible(false);
        content_box.append(&info_label);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_child(Some(&content_box));
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let vert_sep = gtk::Separator::new(gtk::Orientation::Vertical);
        container.append(&vert_sep);
        container.append(&scrolled);

        let revealer = gtk::Revealer::new();
        revealer.set_child(Some(&container));
        revealer.set_reveal_child(false);
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);

        Self {
            widget: revealer,
            content_box,
            title_label,
            image,
            text_view,
            text_scroll,
            info_label,
        }
    }

    pub fn toggle(&self) {
        self.widget.set_reveal_child(!self.widget.reveals_child());
    }

    pub fn is_visible(&self) -> bool {
        self.widget.reveals_child()
    }

    pub fn preview_file(&self, path: &Path) {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        self.title_label.set_text(&name);

        // Reset
        self.image.set_visible(false);
        self.text_scroll.set_visible(false);
        self.info_label.set_visible(false);

        let (content_type, _) = gio::content_type_guess(Some(&name), &[]);
        let ct = content_type.to_string();

        if ct.starts_with("image/") {
            // Image preview
            self.image.set_filename(Some(path));
            self.image.set_visible(true);
        } else if ct.starts_with("text/") || is_text_file(&ct, path) {
            // Text preview (first 5000 chars)
            if let Ok(contents) = std::fs::read_to_string(path) {
                let preview: String = contents.chars().take(5000).collect();
                self.text_view.buffer().set_text(&preview);
                self.text_scroll.set_visible(true);
            } else {
                self.show_info(path, &ct);
            }
        } else {
            self.show_info(path, &ct);
        }
    }

    fn show_info(&self, path: &Path, content_type: &str) {
        let mut info = String::new();
        let desc = gio::content_type_get_description(content_type);
        info.push_str(&format!("Type: {}\n", desc));

        if let Ok(meta) = std::fs::metadata(path) {
            info.push_str(&format!("Size: {}\n", crate::utils::format::format_size(meta.len())));
            if let Ok(modified) = meta.modified() {
                let ts = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                info.push_str(&format!("Modified: {}\n", crate::utils::format::format_date(ts)));
            }
        }

        self.info_label.set_text(&info);
        self.info_label.set_visible(true);
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        self.title_label.set_text("");
        self.image.set_visible(false);
        self.text_scroll.set_visible(false);
        self.info_label.set_visible(false);
    }
}

fn is_text_file(content_type: &str, path: &Path) -> bool {
    // Heuristic: check common text extensions
    let text_types = ["application/json", "application/xml", "application/javascript",
        "application/x-shellscript", "application/toml", "application/yaml",
        "application/x-perl", "application/x-ruby"];
    if text_types.iter().any(|t| content_type.contains(t)) { return true; }

    let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    matches!(ext.as_str(), "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" |
        "java" | "rb" | "sh" | "bash" | "zsh" | "fish" | "md" | "txt" | "log" |
        "toml" | "yaml" | "yml" | "json" | "xml" | "csv" | "ini" | "cfg" |
        "conf" | "html" | "css" | "lua" | "sql" | "vim" | "dockerfile" | "makefile")
}
