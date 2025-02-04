use raw_window_handle::RawWindowHandle;

use crate::platform::{ControlFlow, LogicalSize, WindowEvent};

use std::io;

///////////////////////////////////////////////////////////////////////////////

pub struct Events {
    handle: (),
}

impl Events {
    pub fn run<F>(self, mut _callback: F)
    where
        F: 'static + FnMut(WindowEvent) -> ControlFlow,
    {
        unreachable!()
    }

    pub fn poll(&mut self) -> Vec<WindowEvent> {
        unreachable!()
    }
}

pub struct Window {
    handle: (),
}

impl Window {
    pub fn request_redraw(&self) {
        unreachable!()
    }

    pub fn raw_handle(&self) -> RawWindowHandle {
        unreachable!()
    }

    pub fn set_cursor_visible(&mut self, _visible: bool) {
        unreachable!()
    }

    pub fn hidpi_factor(&self) -> f64 {
        unreachable!()
    }

    pub fn framebuffer_size(&self) -> io::Result<LogicalSize> {
        unreachable!()
    }
}

pub fn init(_title: &str) -> io::Result<(Window, Events)> {
    panic!("no backend found for this platform!");
}
