// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    time::Duration,
};

use glam::Vec3;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::{
    camera::Camera,
    config,
    i18n::{dec, fill, gettext, int, ngettext},
    mesh_info::MeshInfo,
    renderer::Renderer,
    stl,
};

#[derive(Default)]
struct State {
    renderer: Option<Renderer>,
    /// Malla recién cargada que espera ser subida a la GPU en el próximo render.
    pending: Option<stl::Mesh>,
    camera: Camera,
    bounds: Option<(Vec3, Vec3)>,
    wire: bool,
    file: Option<gio::File>,
    monitor: Option<gio::FileMonitor>,
    generation: u64,
    reload_source: Option<glib::SourceId>,
}

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
    stack: gtk::Stack,
    area: gtk::GLArea,
    status: gtk::Label,
    info: InfoLabels,
    state: RefCell<State>,
}

thread_local! {
    static VIEWER: RefCell<Option<Rc<Viewer>>> = const { RefCell::new(None) };
}

/// Ejecuta `f` con el visor si sigue vivo (los closures guardan `Weak` para no formar ciclos).
fn with(w: &Weak<Viewer>, f: impl FnOnce(Rc<Viewer>)) {
    if let Some(v) = w.upgrade() {
        f(v)
    }
}

/// Barra de menús clásica. GTK muestra los atajos al lado de cada opción.
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
    file.append_section(
        None,
        &section(&[(gettext("_Close"), "window.close"), (gettext("_Quit"), "app.quit")]),
    );

    let view = gio::Menu::new();
    view.append_section(
        None,
        &section(&[(gettext("_Wireframe"), "win.wireframe"), (gettext("_Information Panel"), "win.info")]),
    );
    view.append_section(None, &section(&[(gettext("_Fit Model"), "win.fit")]));

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
                        VIEWER.with(|c| c.borrow_mut().take());
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

    fn build(app: &gtk::Application) -> Rc<Self> {
        // Barra de herramientas.
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

        // Área central: vacío / cargando / vista 3D.
        let empty = gtk::Label::new(Some(&gettext(
            "No model open.\n\nUse File → Open (Ctrl+O) or drop an STL file on this window.",
        )));
        empty.set_justify(gtk::Justification::Center);
        empty.add_css_class("dim-label");
        let spinner = gtk::Spinner::builder()
            .spinning(true)
            .width_request(32)
            .height_request(32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        let area = gtk::GLArea::new();
        area.set_allowed_apis(gdk::GLAPI::GL);
        area.set_required_version(3, 3);
        area.set_has_depth_buffer(true);
        area.set_hexpand(true);
        area.set_vexpand(true);
        area.set_focusable(true);

        let stack = gtk::Stack::new();
        stack.set_hexpand(true);
        stack.add_named(&empty, Some("empty"));
        stack.add_named(&spinner, Some("loading"));
        stack.add_named(&area, Some("view"));

        // Panel de información (grilla de etiquetas).
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

        let panel = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&grid)
            .width_request(230)
            .visible(false)
            .build();

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_start_child(Some(&stack));
        paned.set_end_child(Some(&panel));
        paned.set_resize_end_child(false);
        paned.set_shrink_end_child(false);
        paned.set_vexpand(true);

        // Barra de estado.
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
            .default_width(1100)
            .default_height(750)
            .show_menubar(true)
            .child(&vbox)
            .build();

        let this = Rc::new(Self { window, stack, area, status, info, state: RefCell::default() });
        this.setup_actions(&panel);
        this.setup_gl();
        this.setup_input();
        this.setup_drop(&paned);

        // Redibujar si cambia el tema (fondo claro/oscuro).
        if let Some(settings) = gtk::Settings::default() {
            let w = Rc::downgrade(&this);
            settings.connect_gtk_theme_name_notify(move |_| with(&w, |v| v.area.queue_render()));
        }
        this
    }

    fn setup_actions(self: &Rc<Self>, panel: &gtk::ScrolledWindow) {
        let win = &self.window;
        let simple = |name: &str, f: fn(&Rc<Viewer>)| {
            let a = gio::SimpleAction::new(name, None);
            let w = Rc::downgrade(self);
            a.connect_activate(move |_, _| with(&w, |v| f(&v)));
            win.add_action(&a);
        };
        simple("open", |v| v.choose_file());
        simple("reload", |v| v.load(false));
        simple("fit", |v| {
            let mut st = v.state.borrow_mut();
            if let Some((min, max)) = st.bounds {
                st.camera.fit(min, max);
                v.area.queue_render();
            }
        });
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
            let dialog = gtk::AboutDialog::builder()
                .transient_for(&v.window)
                .modal(true)
                .program_name("Trigonum")
                .logo_icon_name(config::APP_ID)
                .version(config::VERSION)
                .comments(gettext("Lightweight viewer for STL 3D models"))
                .website(config::WEBSITE)
                .authors(["nachopro"])
                .license_type(gtk::License::Gpl30)
                .build();
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
                v.state.borrow_mut().wire = on;
                v.area.queue_render();
            });
        });
        win.add_action(&wire);

        win.add_action(&gio::PropertyAction::new("info", panel, "visible"));
    }

    fn setup_gl(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        self.area.connect_realize(move |area| {
            with(&w, |v| {
                area.make_current();
                if let Some(e) = area.error() {
                    v.error(&gettext("OpenGL is not available"), &e.to_string());
                    return;
                }
                match Renderer::new() {
                    Ok(r) => v.state.borrow_mut().renderer = Some(r),
                    Err(e) => v.error(&gettext("Could not initialize OpenGL"), &e),
                }
            })
        });

        let w = Rc::downgrade(self);
        self.area.connect_unrealize(move |area| {
            with(&w, |v| {
                area.make_current();
                if let Some(r) = v.state.borrow_mut().renderer.take() {
                    r.destroy();
                }
            })
        });

        let w = Rc::downgrade(self);
        self.area.connect_render(move |area, _| {
            with(&w, |v| {
                let mut st = v.state.borrow_mut();
                let aspect = area.width().max(1) as f32 / area.height().max(1) as f32;
                let (view, proj) = (st.camera.view(), st.camera.proj(aspect));
                let wire = st.wire;
                let pending = st.pending.take();
                if let Some(r) = st.renderer.as_mut() {
                    if let Some(mesh) = pending {
                        r.upload(&mesh.data);
                    }
                    r.draw(view, proj, wire, theme_is_dark(area));
                }
            });
            glib::Propagation::Stop
        });
    }

    fn setup_input(self: &Rc<Self>) {
        // Arrastre: botón izquierdo orbita; medio, derecho o Shift+izquierdo desplazan.
        let drag = gtk::GestureDrag::new();
        drag.set_button(0);
        let last = Rc::new(Cell::new((0.0f64, 0.0f64)));
        let panning = Rc::new(Cell::new(false));
        {
            let (last, panning) = (last.clone(), panning.clone());
            drag.connect_drag_begin(move |g, _, _| {
                last.set((0.0, 0.0));
                let shift = g.current_event_state().contains(gdk::ModifierType::SHIFT_MASK);
                panning.set(g.current_button() != 1 || shift);
            });
        }
        let w = Rc::downgrade(self);
        drag.connect_drag_update(move |_, ox, oy| {
            with(&w, |v| {
                let (lx, ly) = last.replace((ox, oy));
                let (dx, dy) = ((ox - lx) as f32, (oy - ly) as f32);
                let mut st = v.state.borrow_mut();
                if panning.get() {
                    st.camera.pan(dx, dy, v.area.height() as f32);
                } else {
                    st.camera.orbit(dx, dy);
                }
                v.area.queue_render();
            })
        });
        self.area.add_controller(drag);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let w = Rc::downgrade(self);
        scroll.connect_scroll(move |_, _, dy| {
            with(&w, |v| {
                v.state.borrow_mut().camera.zoom(dy as f32);
                v.area.queue_render();
            });
            glib::Propagation::Stop
        });
        self.area.add_controller(scroll);

        let click = gtk::GestureClick::new();
        let w = Rc::downgrade(self);
        click.connect_pressed(move |_, n, _, _| {
            if n == 2 {
                with(&w, |v| {
                    let _ = WidgetExt::activate_action(&v.window, "win.fit", None);
                })
            }
        });
        self.area.add_controller(click);
    }

    fn setup_drop(self: &Rc<Self>, target_widget: &impl IsA<gtk::Widget>) {
        let target = gtk::DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY);
        let w = Rc::downgrade(self);
        target.connect_drop(move |_, value, _, _| match value.get::<gio::File>() {
            Ok(file) => {
                with(&w, |v| v.open(file));
                true
            }
            Err(_) => false,
        });
        target_widget.add_controller(target);
    }

    fn choose_file(self: &Rc<Self>) {
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
        dialog.open(Some(&self.window), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                with(&w, |v| v.open(file));
            }
        });
    }

    pub fn open(self: &Rc<Self>, file: gio::File) {
        {
            let mut st = self.state.borrow_mut();
            if st.file.as_ref().is_some_and(|f| f.equal(&file)) {
                drop(st);
                return self.load(false);
            }
            st.monitor = None;
            if let Some(id) = st.reload_source.take() {
                id.remove();
            }
            match file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
                Ok(m) => {
                    let w = Rc::downgrade(self);
                    m.connect_changed(move |_, _, _, ev| {
                        if matches!(ev, gio::FileMonitorEvent::ChangesDoneHint | gio::FileMonitorEvent::Created) {
                            with(&w, |v| v.schedule_reload());
                        }
                    });
                    st.monitor = Some(m);
                }
                Err(e) => eprintln!("trigonum: file monitoring unavailable: {e}"),
            }
            st.file = Some(file);
        }
        self.load(true);
    }

    /// Agrupa ráfagas de eventos (p. ej. guardado atómico) en una sola recarga.
    fn schedule_reload(self: &Rc<Self>) {
        let mut st = self.state.borrow_mut();
        if let Some(id) = st.reload_source.take() {
            id.remove();
        }
        let w = Rc::downgrade(self);
        st.reload_source = Some(glib::timeout_add_local_once(Duration::from_millis(250), move || {
            with(&w, |v| {
                v.state.borrow_mut().reload_source = None;
                v.load(false);
            })
        }));
    }

    fn load(self: &Rc<Self>, fresh: bool) {
        let (file, generation) = {
            let mut st = self.state.borrow_mut();
            let Some(file) = st.file.clone() else { return };
            st.generation += 1;
            (file, st.generation)
        };
        let Some(path) = file.path() else {
            return self.error(&gettext("Cannot open file"), &gettext("Only local files are supported."));
        };
        let name = file
            .basename()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        // TRANSLATORS: window title; {name} is the file name.
        self.window.set_title(Some(&fill(gettext("{name} — Trigonum"), &[("name", &name)])));
        self.status.set_text(&fill(gettext("Loading {path}…"), &[("path", &path.display())]));
        if fresh {
            self.stack.set_visible_child_name("loading");
        }

        let started = std::time::Instant::now();
        let handle = gio::spawn_blocking(move || stl::load(&path));
        let w = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let res = handle.await;
            with(&w, |v| {
                if v.state.borrow().generation != generation {
                    return; // llegó una carga más nueva mientras tanto
                }
                match res {
                    Ok(Ok(mesh)) => v.set_mesh(&name, mesh, fresh, started.elapsed()),
                    Ok(Err(e)) => v.load_failed(&name, &e, fresh),
                    Err(_) => v.load_failed(&name, &gettext("internal error while reading"), fresh),
                }
            })
        });
    }

    fn set_mesh(&self, name: &str, mesh: stl::Mesh, fresh: bool, took: Duration) {
        self.show_info(&mesh.info);
        let s = mesh.info.size();
        let n = mesh.info.triangles;
        let triangles = fill(
            ngettext("{count} triangle", "{count} triangles", n.min(u32::MAX as usize) as u32),
            &[("count", &int(n))],
        );
        let template = if fresh {
            // TRANSLATORS: status bar. {name}: file, {triangles}: "N triangles", {size}: "X × Y × Z", {ms}: milliseconds.
            gettext("{name}  ·  {triangles}  ·  {size} mm  ·  loaded in {ms} ms")
        } else {
            // TRANSLATORS: same as the "loaded" message, after the file changed on disk.
            gettext("{name}  ·  {triangles}  ·  {size} mm  ·  reloaded in {ms} ms")
        };
        let size = format!("{} × {} × {}", dec(s.x as f64, 2), dec(s.y as f64, 2), dec(s.z as f64, 2));
        let ms = dec(took.as_secs_f64() * 1000.0, 0);
        self.status.set_text(&fill(
            template,
            &[("name", &name), ("triangles", &triangles), ("size", &size), ("ms", &ms)],
        ));
        let mut st = self.state.borrow_mut();
        let bounds = (mesh.info.min, mesh.info.max);
        if fresh || st.bounds.is_none() {
            st.camera.fit(bounds.0, bounds.1);
        }
        st.bounds = Some(bounds);
        st.pending = Some(mesh);
        drop(st);
        self.stack.set_visible_child_name("view");
        self.area.queue_render();
        self.area.grab_focus();
    }

    fn load_failed(&self, name: &str, err: &str, fresh: bool) {
        self.status.set_text(&format!("{name}: {err}"));
        if fresh {
            let mut st = self.state.borrow_mut();
            st.bounds = None;
            st.pending = Some(stl::Mesh { data: Vec::new(), info: MeshInfo::default() });
            drop(st);
            self.window.set_title(Some("Trigonum"));
            self.stack.set_visible_child_name("empty");
            self.error(&fill(gettext("Could not open “{name}”"), &[("name", &name)]), err);
        }
    }

    fn show_info(&self, info: &MeshInfo) {
        let s = info.size();
        let l = &self.info;
        l.triangles.set_text(&int(info.triangles));
        l.x.set_text(&dec(s.x as f64, 2));
        l.y.set_text(&dec(s.y as f64, 2));
        l.z.set_text(&dec(s.z as f64, 2));
        l.volume.set_text(&format!("{} cm³", dec(info.volume() / 1000.0, 2)));
        l.area.set_text(&format!("{} cm²", dec(info.area / 100.0, 2)));
    }

    fn error(&self, msg: &str, detail: &str) {
        gtk::AlertDialog::builder()
            .message(msg)
            .detail(detail)
            .build()
            .show(Some(&self.window));
    }
}

/// Deduce si el tema GTK actual es oscuro a partir del color de texto del widget.
fn theme_is_dark(w: &impl IsA<gtk::Widget>) -> bool {
    let c = w.color();
    0.299 * c.red() + 0.587 * c.green() + 0.114 * c.blue() > 0.5
}
