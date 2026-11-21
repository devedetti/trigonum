// SPDX-License-Identifier: GPL-3.0-or-later
mod camera;
mod config;
mod i18n;
mod mesh_info;
mod renderer;
mod stl;
mod window;

use gettextrs::{LocaleCategory, bind_textdomain_codeset, bindtextdomain, setlocale, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

fn init_i18n() {
    // SAFETY: se llama al inicio de main, antes de crear otros hilos.
    unsafe { setlocale(LocaleCategory::LcAll, "") };
    let dir = config::locale_dir();
    // Si falla, la interfaz simplemente queda en inglés.
    let _ = bindtextdomain(config::GETTEXT_PACKAGE, dir);
    let _ = bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8");
    let _ = textdomain(config::GETTEXT_PACKAGE);
}

fn main() -> glib::ExitCode {
    init_i18n();
    glib::set_application_name("Trigonum");
    gtk::Window::set_default_icon_name(config::APP_ID);

    let app = gtk::Application::builder()
        .application_id(config::APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        let quit = gio::SimpleAction::new("quit", None);
        let a = app.downgrade();
        quit.connect_activate(move |_, _| {
            if let Some(app) = a.upgrade() {
                app.quit();
            }
        });
        app.add_action(&quit);

        app.set_menubar(Some(&window::menubar()));
        app.set_accels_for_action("win.open", &["<Control>o"]);
        app.set_accels_for_action("win.reload", &["<Control>r", "F5"]);
        app.set_accels_for_action("win.wireframe", &["w"]);
        app.set_accels_for_action("win.fit", &["f"]);
        app.set_accels_for_action("win.info", &["i", "F9"]);
        app.set_accels_for_action("window.close", &["<Control>w"]);
        app.set_accels_for_action("app.quit", &["<Control>q"]);
    });
    app.connect_activate(|app| window::Viewer::get_or_create(app).present());
    app.connect_open(|app, files, _| {
        let v = window::Viewer::get_or_create(app);
        if let Some(f) = files.first() {
            v.open(f.clone());
        }
        v.present();
    });
    app.run()
}
