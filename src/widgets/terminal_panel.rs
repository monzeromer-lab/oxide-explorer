use gtk::prelude::*;
use std::path::Path;

pub struct TerminalPanel {
    pub widget: gtk::Revealer,
    #[cfg(feature = "terminal")]
    terminal: vte4::Terminal,
}

impl TerminalPanel {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        header.set_margin_start(8);
        header.set_margin_end(4);

        let label = gtk::Label::new(Some("Terminal"));
        label.add_css_class("heading");
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        header.append(&label);

        let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        close_btn.set_tooltip_text(Some("Hide Terminal"));
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

            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            term.spawn_async(
                vte4::PtyFlags::DEFAULT,
                None,
                &[&shell],
                &[],
                glib::SpawnFlags::DEFAULT,
                || {},
                -1,
                gio::Cancellable::NONE,
                |_terminal, _result| {},
            );

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
            placeholder.set_description(Some("Build with --features terminal\n(requires libvte-2.91-gtk4-dev)"));
            placeholder.set_vexpand(true);
            placeholder.set_height_request(200);
            container.append(&placeholder);
        }

        let revealer = gtk::Revealer::new();
        revealer.set_child(Some(&container));
        revealer.set_reveal_child(false);
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);

        let rev = revealer.clone();
        close_btn.connect_clicked(move |_| { rev.set_reveal_child(false); });

        Self {
            widget: revealer,
            #[cfg(feature = "terminal")]
            terminal,
        }
    }

    pub fn toggle(&self) {
        let visible = self.widget.reveals_child();
        self.widget.set_reveal_child(!visible);
        #[cfg(feature = "terminal")]
        if !visible {
            self.terminal.grab_focus();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.widget.reveals_child()
    }

    pub fn cd(&self, path: &Path) {
        #[cfg(feature = "terminal")]
        {
            let cmd = format!("cd {}\n", shell_escape(path));
            self.terminal.feed_child(cmd.as_bytes());
        }
    }
}

fn shell_escape(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("'{}'", s.replace('\'', "'\\''"))
}
