use gtk::prelude::*;
#[cfg(feature = "terminal")]
use vte4::prelude::*;
use std::path::Path;

pub struct TerminalPanel {
    pub widget: gtk::Box,
    pub close_btn: gtk::Button,
    #[cfg(feature = "terminal")]
    terminal: vte4::Terminal,
    #[cfg(feature = "terminal")]
    spawned: std::cell::Cell<bool>,
}

impl TerminalPanel {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        header.set_margin_start(8);
        header.set_margin_end(4);
        header.set_margin_top(2);
        header.set_margin_bottom(2);
        header.add_css_class("terminal-header");

        let label = gtk::Label::new(Some("Terminal"));
        label.add_css_class("heading");
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        header.append(&label);

        let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        close_btn.set_tooltip_text(Some("Hide Terminal (F4)"));
        header.append(&close_btn);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        container.append(&sep);
        container.append(&header);

        #[cfg(feature = "terminal")]
        let terminal = {
            let term = vte4::Terminal::new();
            term.set_size_request(-1, 250);
            term.set_scroll_on_output(true);
            term.set_scrollback_lines(10000);

            let scrolled = gtk::ScrolledWindow::new();
            scrolled.set_child(Some(&term));
            scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
            container.append(&scrolled);
            term
        };

        #[cfg(not(feature = "terminal"))]
        {
            let placeholder = adw::StatusPage::new();
            placeholder.set_icon_name(Some("utilities-terminal-symbolic"));
            placeholder.set_title("Terminal Not Available");
            placeholder.set_description(Some(
                "Build with --features terminal\n(requires libvte-2.91-gtk4-dev)",
            ));
            placeholder.set_vexpand(true);
            placeholder.set_height_request(200);
            container.append(&placeholder);
        }

        Self {
            widget: container,
            close_btn,
            #[cfg(feature = "terminal")]
            terminal,
            #[cfg(feature = "terminal")]
            spawned: std::cell::Cell::new(false),
        }
    }

    /// Spawn the shell at the given directory. Only spawns once; subsequent calls use cd.
    pub fn spawn_at(&self, cwd: &Path) {
        #[cfg(feature = "terminal")]
        {
            if self.spawned.get() {
                self.cd(cwd);
                return;
            }
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            let cwd_str = cwd.to_string_lossy().to_string();
            self.terminal.spawn_async(
                vte4::PtyFlags::DEFAULT,
                Some(&cwd_str),
                &[&shell],
                &[],
                glib::SpawnFlags::DEFAULT,
                || {},
                -1,
                gio::Cancellable::NONE,
                |_result| {},
            );
            self.spawned.set(true);
        }
    }

    pub fn focus(&self) {
        #[cfg(feature = "terminal")]
        self.terminal.grab_focus();
    }

    pub fn cd(&self, path: &Path) {
        #[cfg(feature = "terminal")]
        {
            if !self.spawned.get() {
                self.spawn_at(path);
                return;
            }
            let cmd = format!("cd {} && clear\n", shell_escape(path));
            self.terminal.feed_child(cmd.as_bytes());
        }
    }
}

fn shell_escape(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("'{}'", s.replace('\'', "'\\''"))
}
