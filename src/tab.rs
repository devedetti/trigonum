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
    i18n::{dec, fill, gettext, int, ngettext},
    mesh_info::MeshInfo,
    renderer::Renderer,
    stl,
    window::{Viewer, with},
};

pub struct TabState {
    renderer: Option<Renderer>,
    pending: Option<stl::Mesh>,
    camera: Camera,
    bounds: Option<(Vec3, Vec3)>,
    pub wire: bool,
    pub file: gio::File,
    monitor: Option<gio::FileMonitor>,
    generation: u64,
    reload_source: Option<glib::SourceId>,
    pub name: String,
    pub status: String,
    pub info: Option<MeshInfo>,
    loaded: bool,
    stale: bool,
}

pub struct Tab {
    viewer: Weak<Viewer>,
    pub page: gtk::Stack,
    pub label: gtk::Box,
    area: gtk::GLArea,
    pub state: RefCell<TabState>,
}

impl Tab {
    pub fn new(viewer: &Rc<Viewer>, file: gio::File) -> Rc<Self> {
        let name = file
            .basename()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let path = file.path().map(|p| p.display().to_string()).unwrap_or_else(|| file.uri().into());

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

        let page = gtk::Stack::new();
        page.add_named(&spinner, Some("loading"));
        page.add_named(&area, Some("view"));
        page.set_visible_child_name("loading");

        let title = gtk::Label::builder()
            .label(&name)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .width_chars(name.chars().count().min(28) as i32)
            .max_width_chars(28)
            .build();
        let close = gtk::Button::from_icon_name("window-close-symbolic");
        close.add_css_class("flat");
        close.set_focus_on_click(false);
        close.set_tooltip_text(Some(&gettext("Close tab")));
        let label = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        label.set_tooltip_text(Some(&path));
        label.append(&title);
        label.append(&close);

        let this = Rc::new(Self {
            viewer: Rc::downgrade(viewer),
            page,
            label,
            area,
            state: RefCell::new(TabState {
                renderer: None,
                pending: None,
                camera: Camera::default(),
                bounds: None,
                wire: false,
                file,
                monitor: None,
                generation: 0,
                reload_source: None,
                name,
                status: String::new(),
                info: None,
                loaded: false,
                stale: false,
            }),
        });

        let w = Rc::downgrade(&this);
        close.connect_clicked(move |_| with(&w, |t| t.close()));
        let middle = gtk::GestureClick::new();
        middle.set_button(2);
        let w = Rc::downgrade(&this);
        middle.connect_released(move |_, _, _, _| with(&w, |t| t.close()));
        this.label.add_controller(middle);

        this.setup_gl();
        this.setup_input();
        this.setup_monitor();
        this
    }

    fn close(self: &Rc<Self>) {
        if let Some(v) = self.viewer.upgrade() {
            v.close_tab(self);
        }
    }

    fn is_current(&self) -> bool {
        self.viewer.upgrade().is_some_and(|v| v.is_current(self))
    }

    pub fn shutdown(&self) {
        let mut st = self.state.borrow_mut();
        if let Some(m) = st.monitor.take() {
            m.cancel();
        }
        if let Some(id) = st.reload_source.take() {
            id.remove();
        }
    }

    pub fn activated(self: &Rc<Self>) {
        let (loaded, stale) = {
            let st = self.state.borrow();
            (st.loaded, st.stale)
        };
        if !loaded {
            self.load(true);
        } else if stale {
            self.load(false);
        }
        self.area.grab_focus();
    }

    pub fn reload(self: &Rc<Self>) {
        let loaded = self.state.borrow().loaded;
        self.load(!loaded);
    }

    pub fn mark_stale(&self) {
        let mut st = self.state.borrow_mut();
        if st.loaded {
            st.stale = true;
        }
    }

    pub fn fit(&self) {
        let mut st = self.state.borrow_mut();
        if let Some((min, max)) = st.bounds {
            st.camera.fit(min, max);
            self.area.queue_render();
        }
    }

    pub fn set_wire(&self, on: bool) {
        self.state.borrow_mut().wire = on;
        self.area.queue_render();
    }

    pub fn queue_render(&self) {
        self.area.queue_render();
    }

    fn setup_gl(self: &Rc<Self>) {
        let w = Rc::downgrade(self);
        self.area.connect_realize(move |area| {
            with(&w, |t| {
                area.make_current();
                let Some(v) = t.viewer.upgrade() else { return };
                if let Some(e) = area.error() {
                    v.error(&gettext("OpenGL is not available"), &e.to_string());
                    return;
                }
                match Renderer::new() {
                    Ok(r) => t.state.borrow_mut().renderer = Some(r),
                    Err(e) => v.error(&gettext("Could not initialize OpenGL"), &e),
                }
            })
        });

        let w = Rc::downgrade(self);
        self.area.connect_unrealize(move |area| {
            with(&w, |t| {
                area.make_current();
                if let Some(r) = t.state.borrow_mut().renderer.take() {
                    r.destroy();
                }
            })
        });

        let w = Rc::downgrade(self);
        self.area.connect_render(move |area, _| {
            with(&w, |t| {
                let mut st = t.state.borrow_mut();
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
            with(&w, |t| {
                let (lx, ly) = last.replace((ox, oy));
                let (dx, dy) = ((ox - lx) as f32, (oy - ly) as f32);
                let mut st = t.state.borrow_mut();
                if panning.get() {
                    st.camera.pan(dx, dy, t.area.height() as f32);
                } else {
                    st.camera.orbit(dx, dy);
                }
                t.area.queue_render();
            })
        });
        self.area.add_controller(drag);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let w = Rc::downgrade(self);
        scroll.connect_scroll(move |_, _, dy| {
            with(&w, |t| {
                t.state.borrow_mut().camera.zoom(dy as f32);
                t.area.queue_render();
            });
            glib::Propagation::Stop
        });
        self.area.add_controller(scroll);

        let click = gtk::GestureClick::new();
        let w = Rc::downgrade(self);
        click.connect_pressed(move |_, n, _, _| {
            if n == 2 {
                with(&w, |t| t.fit())
            }
        });
        self.area.add_controller(click);
    }

    fn setup_monitor(self: &Rc<Self>) {
        let mut st = self.state.borrow_mut();
        match st.file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(m) => {
                let w = Rc::downgrade(self);
                m.connect_changed(move |_, _, _, ev| {
                    if matches!(ev, gio::FileMonitorEvent::ChangesDoneHint | gio::FileMonitorEvent::Created) {
                        with(&w, |t| {
                            if t.is_current() {
                                t.schedule_reload()
                            } else {
                                t.mark_stale()
                            }
                        });
                    }
                });
                st.monitor = Some(m);
            }
            Err(e) => eprintln!("trigonum: file monitoring unavailable: {e}"),
        }
    }

    fn schedule_reload(self: &Rc<Self>) {
        let mut st = self.state.borrow_mut();
        if let Some(id) = st.reload_source.take() {
            id.remove();
        }
        let w = Rc::downgrade(self);
        st.reload_source = Some(glib::timeout_add_local_once(Duration::from_millis(250), move || {
            with(&w, |t| {
                t.state.borrow_mut().reload_source = None;
                if t.is_current() {
                    t.load(false);
                } else {
                    t.mark_stale();
                }
            })
        }));
    }

    fn set_status(&self, text: String) {
        if self.is_current() {
            if let Some(v) = self.viewer.upgrade() {
                v.show_status(&text);
            }
        }
        self.state.borrow_mut().status = text;
    }

    fn load(self: &Rc<Self>, fresh: bool) {
        let (file, name, generation) = {
            let mut st = self.state.borrow_mut();
            st.loaded = true;
            st.stale = false;
            if let Some(id) = st.reload_source.take() {
                id.remove();
            }
            st.generation += 1;
            (st.file.clone(), st.name.clone(), st.generation)
        };
        let Some(path) = file.path() else {
            return self.load_failed(&name, &gettext("Only local files are supported."), fresh);
        };
        self.set_status(fill(gettext("Loading {path}…"), &[("path", &path.display())]));
        if fresh {
            self.page.set_visible_child_name("loading");
        }

        let started = std::time::Instant::now();
        let handle = gio::spawn_blocking(move || stl::load(&path));
        let w = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let res = handle.await;
            with(&w, |t| {
                if t.state.borrow().generation != generation {
                    return;
                }
                match res {
                    Ok(Ok(mesh)) => t.set_mesh(&name, mesh, fresh, started.elapsed()),
                    Ok(Err(e)) => t.load_failed(&name, &e, fresh),
                    Err(_) => t.load_failed(&name, &gettext("internal error while reading"), fresh),
                }
            })
        });
    }

    fn set_mesh(&self, name: &str, mesh: stl::Mesh, fresh: bool, took: Duration) {
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
        self.set_status(fill(
            template,
            &[("name", &name), ("triangles", &triangles), ("size", &size), ("ms", &ms)],
        ));
        if self.is_current() {
            if let Some(v) = self.viewer.upgrade() {
                v.show_info(Some(&mesh.info));
            }
        }
        let mut st = self.state.borrow_mut();
        let bounds = (mesh.info.min, mesh.info.max);
        if fresh || st.bounds.is_none() {
            st.camera.fit(bounds.0, bounds.1);
        }
        st.bounds = Some(bounds);
        st.info = Some(mesh.info.clone());
        st.pending = Some(mesh);
        drop(st);
        self.page.set_visible_child_name("view");
        self.area.queue_render();
        if self.is_current() {
            self.area.grab_focus();
        }
    }

    fn load_failed(self: &Rc<Self>, name: &str, err: &str, fresh: bool) {
        self.set_status(format!("{name}: {err}"));
        if fresh {
            if let Some(v) = self.viewer.upgrade() {
                v.close_tab(self);
                v.error(&fill(gettext("Could not open “{name}”"), &[("name", &name)]), err);
            }
        }
    }
}

fn theme_is_dark(w: &impl IsA<gtk::Widget>) -> bool {
    let c = w.color();
    0.299 * c.red() + 0.587 * c.green() + 0.114 * c.blue() > 0.5
}
