// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use gtk::glib;

const GROUP: &str = "window";
const MIN_WIDTH: i32 = 400;
const MIN_HEIGHT: i32 = 300;

pub struct WindowState {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub info_panel: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self { width: 1100, height: 750, maximized: false, info_panel: false }
    }
}

fn path() -> PathBuf {
    glib::user_config_dir().join("trigonum").join("window.ini")
}

impl WindowState {
    pub fn load() -> Self {
        let mut s = Self::default();
        let kf = glib::KeyFile::new();
        if kf.load_from_file(path(), glib::KeyFileFlags::NONE).is_err() {
            return s;
        }
        if let Ok(v) = kf.integer(GROUP, "width") {
            s.width = v.max(MIN_WIDTH);
        }
        if let Ok(v) = kf.integer(GROUP, "height") {
            s.height = v.max(MIN_HEIGHT);
        }
        if let Ok(v) = kf.boolean(GROUP, "maximized") {
            s.maximized = v;
        }
        if let Ok(v) = kf.boolean(GROUP, "info-panel") {
            s.info_panel = v;
        }
        s
    }

    pub fn save(&self) {
        let kf = glib::KeyFile::new();
        kf.set_integer(GROUP, "width", self.width);
        kf.set_integer(GROUP, "height", self.height);
        kf.set_boolean(GROUP, "maximized", self.maximized);
        kf.set_boolean(GROUP, "info-panel", self.info_panel);
        let p = path();
        let res = p
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .map_err(|e| e.to_string())
            .and_then(|()| kf.save_to_file(&p).map_err(|e| e.to_string()));
        if let Err(e) = res {
            eprintln!("trigonum: could not save {}: {e}", p.display());
        }
    }
}
