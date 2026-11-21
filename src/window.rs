// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::{
    config,
    i18n::{dec, fill, gettext, int},
    mesh_info::MeshInfo,
    tab::Tab,
    window_state::WindowState,
};

struct InfoLabels {
    triangles: gtk::Label,
    x: gtk::Label,
    y: gtk::Label,
    z: gtk::Label,
    volume: gtk::Label,
    area: gtk::Label,
}

pub struct Viewer {
    window: gtk::ApplicationWindow,
    outer: gtk::Stack,
    notebook: gtk::Notebook,
    status: gtk::Label,
    info: InfoLabels,
    panel: gtk::ScrolledWindow,
    tabs: RefCell<Vec<Rc<Tab>>>,
}

thread_local! {
    static VIEWER: RefCell<Option<Rc<Viewer>>> = const { RefCell::new(None) };
}

pub fn with<T>(w: &Weak<T>, f: impl FnOnce(Rc<T>)) {
    if let Some(v) = w.upgrade() {
        f(v)
    }
}

const TAB_ACTIONS: &[&str] = &["reload", "fit", "wireframe", "close-tab", "next-tab", "prev-tab", "goto-tab"];

pub fn menubar() -> gio::Menu {
    let section = |items: &[(String, &str)]| {
        let m = gio::Menu::new();
        for (label, action) in items {
            m.append(Some(label), Some(action));
        }
        m
    };
    let file = gio::Menu::new();
    file.append_section(
        None,
        &section(&[(gettext("_Open…"), "win.open"), (gettext("_Reload"), "win.reload")]),
    );
    file.append_section(None, &section(&[(gettext("_Close Tab"), "win.close-tab")]));
    file.append_section(None, &section(&[(gettext("_Quit"), "app.quit")]));

    let view = gio::Menu::new();
    view.append_section(
        None,
        &section(&[(gettext("_Wireframe"), "win.wireframe"), (gettext("_Information Panel"), "win.info")]),
    );
    view.append_section(None, &section(&[(gettext("_Fit Model"), "win.fit")]));
    view.append_section(
        None,
        &section(&[(gettext("_Next Tab"), "win.next-tab"), (gettext("_Previous Tab"), "win.prev-tab")]),
    );

    let help = section(&[(gettext("_Mouse Controls"), "win.controls"), (gettext("_About Trigonum"), "win.about")]);

    let bar = gio::Menu::new();
    bar.append_submenu(Some(&gettext("_File")), &file);
    bar.append_submenu(Some(&gettext("_View")), &view);
    bar.append_submenu(Some(&gettext("_Help")), &help);
    bar
}

fn tool_button(icon: Option<&str>, label: &str, tip: &str, action: &str, toggle: bool) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    if let Some(icon) = icon {
        content.append(&gtk::Image::from_icon_name(icon));
    }
    content.append(&gtk::Label::new(Some(label)));
    let b: gtk::Button = if toggle {
        gtk::ToggleButton::new().upcast()
    } else {
        gtk::Button::new()
    };
    b.set_child(Some(&content));
    b.set_tooltip_text(Some(tip));
    b.set_action_name(Some(action));
    b.add_css_class("flat");
    b.upcast()
}

impl Viewer {
    pub fn get_or_create(app: &gtk::Application) -> Rc<Self> {
        VIEWER.with(|cell| {
            cell.borrow_mut()
                .get_or_insert_with(|| {
                    let v = Self::build(app);
                    v.window.connect_close_request(|_| {
                        if let Some(v) = VIEWER.with(|c| c.borrow_mut().take()) {
                            v.save_state();
                        }
                        glib::Propagation::Proceed
                    });
                    v
                })
                .clone()
        })
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn save_state(&self) {
        let (width, height) = self.window.default_size();
        WindowState {
            width,
            height,
            maximized: self.window.is_maximized(),
            info_panel: self.panel.is_visible(),
        }
        .save();
    }

    fn build(app: &gtk::Application) -> Rc<Self> {
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        toolbar.add_css_class("toolbar");
        let tb = |icon, label: String, tip: String, action, toggle| {
            toolbar.append(&tool_button(icon, &label, &tip, action, toggle))
        };
        tb(Some("document-open"), gettext("Open"), gettext("Open a file (Ctrl+O)"), "win.open", false);
        tb(Some("view-refresh"), gettext("Reload"), gettext("Reload the model (Ctrl+R)"), "win.reload", false);
        toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        tb(Some("zoom-fit-best"), gettext("Fit"), gettext("Fit the model in view (F)"), "win.fit", false);
        tb(None, gettext("Wireframe"), gettext("Show mesh edges (W)"), "win.wireframe", true);
        tb(Some("dialog-information"), gettext("Information"), gettext("Show model information (I)"), "win.info", true);

        let empty = gtk::Label::new(Some(&gettext(
            "No model open.\n\nUse File → Open (Ctrl+O) or drop an STL file on this window.",
        )));
        empty.set_justify(gtk::Justification::Center);
        empty.add_css_class("dim-label");
        let notebook = gtk::Notebook::builder().scrollable(true).show_border(false).build();

        let outer = gtk::Stack::new();
        outer.set_hexpand(true);
        outer.add_named(&empty, Some("empty"));
        outer.add_named(&notebook, Some("tabs"));

        let grid = gtk::Grid::builder()
            .row_spacing(6)
            .column_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        let heading = |grid: &gtk::Grid, row: i32, text: &str| {
            let l = gtk::Label::builder().label(text).xalign(0.0).margin_top(if row == 0 { 0 } else { 8 }).build();
            l.add_css_class("heading");
            grid.attach(&l, 0, row, 2, 1);
            row + 1
        };
        let field = |grid: &gtk::Grid, row: i32, name: &str| {
            let n = gtk::Label::builder().label(name).xalign(0.0).build();
            n.add_css_class("dim-label");
            let v = gtk::Label::builder().label("—").xalign(1.0).hexpand(true).selectable(true).build();
            v.add_css_class("numeric");
            grid.attach(&n, 0, row, 1, 1);
            grid.attach(&v, 1, row, 1, 1);
            v
        };
        let r = heading(&grid, 0, &gettext("Model"));
        let triangles = field(&grid, r, &gettext("Triangles"));
        let volume = field(&grid, r + 1, &gettext("Volume"));
        let area_l = field(&grid, r + 2, &gettext("Surface area"));
        let r = heading(&grid, r + 3, &gettext("Dimensions (mm)"));
        let x = field(&grid, r, &gettext("Width (X)"));
        let y = field(&grid, r + 1, &gettext("Depth (Y)"));
        let z = field(&grid, r + 2, &gettext("Height (Z)"));
        let info = InfoLabels { triangles, x, y, z, volume, area: area_l };

        let saved = WindowState::load();
        let panel = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&grid)
            .width_request(230)
            .visible(saved.info_panel)
            .build();

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_start_child(Some(&outer));
        paned.set_end_child(Some(&panel));
        paned.set_resize_end_child(false);
        paned.set_shrink_end_child(false);
        paned.set_vexpand(true);

        let status = gtk::Label::builder()
            .label(gettext("Ready"))
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .margin_start(6)
            .margin_end(6)
            .margin_top(3)
            .margin_bottom(3)
            .build();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.append(&toolbar);
        vbox.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        vbox.append(&paned);
        vbox.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        vbox.append(&status);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Trigonum")
            .icon_name(config::APP_ID)
            .default_width(saved.width)
            .default_height(saved.height)
            .maximized(saved.maximized)
            .show_menubar(true)
            .child(&vbox)
            .build();

        let this = Rc::new(Self { window, outer, notebook, status, info, panel, tabs: RefCell::default() });
        this.setup_actions();
        this.setup_drop(&paned);

        let w = Rc::downgrade(&this);
        this.notebook.connect_page_notify(move |_| with(&w, |v| v.refresh()));

        if let Some(settings) = gtk::Settings::default() {
            let w = Rc::downgrade(&this);
            settings.connect_gtk_theme_name_notify(move |_| {
                with(&w, |v| {
                    if let Some(t) = v.current_tab() {
                        t.queue_render();
                    }
                })
            });
        }
        this.refresh();
        this
    }

    fn setup_actions(self: &Rc<Self>) {
        let win = &self.window;
        let simple = |name: &str, f: fn(&Rc<Viewer>)| {
            let a = gio::SimpleAction::new(name, None);
            let w = Rc::downgrade(self);
            a.connect_activate(move |_, _| with(&w, |v| f(&v)));
            win.add_action(&a);
        };
        simple("open", |v| v.choose_files());
        simple("reload", |v| {
            if let Some(t) = v.current_tab() {
                t.reload();
            }
        });
        simple("fit", |v| {
            if let Some(t) = v.current_tab() {
                t.fit();
            }
        });
        simple("close-tab", |v| {
            if let Some(t) = v.current_tab() {
                v.close_tab(&t);
            }
        });
        simple("next-tab", |v| v.cycle_tab(1));
        simple("prev-tab", |v| v.cycle_tab(-1));

        let goto = gio::SimpleAction::new("goto-tab", Some(glib::VariantTy::INT32));
        let w = Rc::downgrade(self);
        goto.connect_activate(move |_, param| {
            let Some(n) = param.and_then(|p| p.get::<i32>()) else { return };
            with(&w, |v| {
                let pages = v.notebook.n_pages();
                if pages > 0 {
                    let idx = if n < 0 { pages - 1 } else { n as u32 };
                    if idx < pages {
                        v.notebook.set_current_page(Some(idx));
                    }
                }
            })
        });
        win.add_action(&goto);
        simple("controls", |v| {
            gtk::AlertDialog::builder()
                .message(gettext("Mouse Controls"))
                .detail(gettext(
                    "Left button + drag: orbit\nMiddle or right button + drag: pan\nShift + left button + drag: pan\nScroll wheel: zoom\nDouble click: fit model",
                ))
                .build()
                .show(Some(&v.window));
        });
        simple("about", |v| {
            // TRANSLATORS: replace with your name(s), one per line, e.g. "Jane Doe <jane@example.com>".
            let credits = gettext("translator-credits");
            let comments = format!(
                "{}\n\n{}",
                gettext("Lightweight viewer for STL 3D models"),
                gettext(
                    "Made so you don't have to rely on JavaScript or intrusive websites: simple and easy to \
                     use. Available for Debian, Ubuntu, Fedora, openSUSE and Arch.",
                ),
            );
            let dialog = gtk::AboutDialog::builder()
                .transient_for(&v.window)
                .modal(true)
                .program_name("Trigonum")
                .logo_icon_name(config::APP_ID)
                .version(config::VERSION)
                .comments(comments)
                .website(config::REPOSITORY)
                .website_label(gettext("Repository"))
                // TRANSLATORS: credits page, under "Created by".
                .authors([gettext("nachopro, from Argentina")])
                .license_type(gtk::License::Gpl30)
                .build();
            let entry = |text: &str, url: &str| format!("{text} {}", url.replace('&', "&amp;"));
            dialog.add_credit_section(&gettext("Binaries for distros"), &[&entry("openSUSE repo", config::DOWNLOADS)]);
            dialog.add_credit_section(&gettext("Issues and PRs"), &[&entry("GitHub", config::WEBSITE)]);
            if credits != "translator-credits" {
                dialog.set_translator_credits(Some(&credits));
            }
            dialog.present();
        });

        let wire = gio::SimpleAction::new_stateful("wireframe", None, &false.to_variant());
        let w = Rc::downgrade(self);
        wire.connect_activate(move |a, _| {
            let on = !a.state().and_then(|s| s.get::<bool>()).unwrap_or(false);
            a.set_state(&on.to_variant());
            with(&w, |v| {
                if let Some(t) = v.current_tab() {
                    t.set_wire(on);
                }
            });
        });
        win.add_action(&wire);

        win.add_action(&gio::PropertyAction::new("info", &self.panel, "visible"));
    }

    fn action(&self, name: &str) -> Option<gio::SimpleAction> {
        self.window.lookup_action(name).and_downcast::<gio::SimpleAction>()
    }

    fn setup_drop(self: &Rc<Self>, target_widget: &impl IsA<gtk::Widget>) {
        let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        let w = Rc::downgrade(self);
        target.connect_drop(move |_, value, _, _| match value.get::<gdk::FileList>() {
            Ok(list) if !list.files().is_empty() => {
                with(&w, |v| v.open_files(&list.files()));
                true
            }
            _ => false,
        });
        target_widget.add_controller(target);
    }

    fn choose_files(self: &Rc<Self>) {
        let stl = gtk::FileFilter::new();
        stl.set_name(Some(&gettext("STL models")));
        stl.add_suffix("stl");
        stl.add_mime_type("model/stl");
        let all = gtk::FileFilter::new();
        all.set_name(Some(&gettext("All files")));
        all.add_pattern("*");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&stl);
        filters.append(&all);
        let dialog = gtk::FileDialog::builder()
            .title(gettext("Open STL Model"))
            .filters(&filters)
            .default_filter(&stl)
            .build();
        let w = Rc::downgrade(self);
        dialog.open_multiple(Some(&self.window), gio::Cancellable::NONE, move |res| {
            if let Ok(list) = res {
                let files: Vec<gio::File> =
                    (0..list.n_items()).filter_map(|i| list.item(i).and_downcast::<gio::File>()).collect();
                with(&w, |v| v.open_files(&files));
            }
        });
    }

    pub fn open_files(self: &Rc<Self>, files: &[gio::File]) {
        for (i, f) in files.iter().enumerate() {
            self.open(f.clone(), i == 0);
        }
    }

    fn open(self: &Rc<Self>, file: gio::File, focus: bool) {
        let existing = self.tabs.borrow().iter().find(|t| t.state.borrow().file.equal(&file)).cloned();
        let tab = match existing {
            Some(t) => {
                t.mark_stale();
                t
            }
            None => {
                let t = Tab::new(self, file);
                self.tabs.borrow_mut().push(t.clone());
                self.notebook.append_page(&t.page, Some(&t.label));
                self.notebook.set_tab_reorderable(&t.page, true);
                t
            }
        };
        if focus {
            self.notebook.set_current_page(self.notebook.page_num(&tab.page));
        }
        self.refresh();
    }

    pub fn close_tab(self: &Rc<Self>, tab: &Rc<Tab>) {
        tab.shutdown();
        if let Some(n) = self.notebook.page_num(&tab.page) {
            self.notebook.remove_page(Some(n));
        }
        self.tabs.borrow_mut().retain(|t| !Rc::ptr_eq(t, tab));
        self.refresh();
    }

    fn cycle_tab(&self, step: i32) {
        let n = self.notebook.n_pages() as i32;
        if let Some(cur) = self.notebook.current_page() {
            if n > 1 {
                self.notebook.set_current_page(Some((cur as i32 + step).rem_euclid(n) as u32));
            }
        }
    }

    fn current_tab(&self) -> Option<Rc<Tab>> {
        let page = self.notebook.nth_page(self.notebook.current_page())?;
        self.tabs.borrow().iter().find(|t| t.page.upcast_ref::<gtk::Widget>() == &page).cloned()
    }

    pub fn is_current(&self, tab: &Tab) -> bool {
        self.current_tab().is_some_and(|t| std::ptr::eq(&*t, tab))
    }

    fn refresh(&self) {
        let tab = self.current_tab();
        for name in TAB_ACTIONS {
            if let Some(a) = self.action(name) {
                a.set_enabled(tab.is_some());
            }
        }
        let wire = self.action("wireframe");
        let Some(tab) = tab else {
            self.outer.set_visible_child_name("empty");
            self.window.set_title(Some("Trigonum"));
            self.status.set_text(&gettext("Ready"));
            self.show_info(None);
            if let Some(a) = wire {
                a.set_state(&false.to_variant());
            }
            return;
        };
        self.outer.set_visible_child_name("tabs");
        {
            let st = tab.state.borrow();
            // TRANSLATORS: window title; {name} is the file name.
            self.window.set_title(Some(&fill(gettext("{name} — Trigonum"), &[("name", &st.name)])));
            self.status.set_text(&st.status);
            self.show_info(st.info.as_ref());
            if let Some(a) = wire {
                a.set_state(&st.wire.to_variant());
            }
        }
        tab.activated();
    }

    pub fn show_status(&self, text: &str) {
        self.status.set_text(text);
    }

    pub fn show_info(&self, info: Option<&MeshInfo>) {
        let l = &self.info;
        let Some(info) = info else {
            for label in [&l.triangles, &l.x, &l.y, &l.z, &l.volume, &l.area] {
                label.set_text("—");
            }
            return;
        };
        let s = info.size();
        l.triangles.set_text(&int(info.triangles));
        l.x.set_text(&dec(s.x as f64, 2));
        l.y.set_text(&dec(s.y as f64, 2));
        l.z.set_text(&dec(s.z as f64, 2));
        l.volume.set_text(&format!("{} cm³", dec(info.volume() / 1000.0, 2)));
        l.area.set_text(&format!("{} cm²", dec(info.area / 100.0, 2)));
    }

    pub fn error(&self, msg: &str, detail: &str) {
        gtk::AlertDialog::builder()
            .message(msg)
            .detail(detail)
            .build()
            .show(Some(&self.window));
    }
}
