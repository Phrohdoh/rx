#![deny(clippy::all)]
mod brush;
mod cmd;
mod font;
mod framebuffer2d;
mod gpu;
mod palette;
mod platform;
mod renderer;
mod resources;
mod screen2d;
mod session;
mod view;

use renderer::Renderer;
use resources::ResourceManager;
use session::*;
use view::FileStatus;

use rgx;
use rgx::core;
use rgx::kit;
use rgx::kit::shape2d;

#[macro_use]
extern crate log;

use env_logger;

use xdg;

use std::collections::VecDeque;
use std::path::Path;
use std::time;

pub struct FrameTimer {
    timings: VecDeque<u128>,
    avg: time::Duration,
}

impl FrameTimer {
    const WINDOW: usize = 60;

    pub fn new() -> Self {
        Self {
            timings: VecDeque::with_capacity(Self::WINDOW),
            avg: time::Duration::from_secs(0),
        }
    }

    pub fn run<F>(&mut self, frame: F)
    where
        F: FnOnce(time::Duration),
    {
        let start = time::Instant::now();
        frame(self.avg);
        let elapsed = start.elapsed();

        self.timings.truncate(Self::WINDOW - 1);
        self.timings.push_front(elapsed.as_micros());

        let avg =
            self.timings.iter().sum::<u128>() / self.timings.len() as u128;
        self.avg = time::Duration::from_micros(avg as u64);
    }
}

pub fn init<P: AsRef<Path>>(paths: &[P]) -> std::io::Result<()> {
    let mut logger = env_logger::Builder::from_default_env();
    logger.init();

    let (mut win, events) = platform::init("rx")?;

    let hidpi_factor = win.hidpi_factor();
    let win_size = win.framebuffer_size()?;
    let (win_w, win_h) = (win_size.width as u32, win_size.height as u32);

    let resources = ResourceManager::new();
    let base_dirs = xdg::BaseDirectories::with_prefix("rx")?;
    let mut session =
        Session::new(win_w, win_h, hidpi_factor, resources.clone(), base_dirs)
            .init()?;

    let mut present_mode = session.settings.present_mode();
    let mut r = core::Renderer::new(win.raw_handle());
    let mut renderer = Renderer::new(&mut r, win_w, win_h, resources);

    if let Err(e) = session.edit(paths) {
        session.message(
            format!("Error loading path(s): {}", e),
            MessageType::Error,
        );
    }
    if session.views.is_empty() {
        session.blank(
            FileStatus::NoFile,
            Session::DEFAULT_VIEW_W,
            Session::DEFAULT_VIEW_H,
        );
    }

    renderer.init(&session, &mut r);

    let physical = win_size.to_physical(hidpi_factor);
    let mut swap_chain = r.swap_chain(
        physical.width as u32,
        physical.height as u32,
        present_mode,
    );

    let mut render_timer = FrameTimer::new();
    let mut canvas = shape2d::Batch::new();
    let mut session_events = Vec::with_capacity(16);
    let mut last = time::Instant::now();

    events.run(move |event| {
        match event {
            platform::WindowEvent::Resized(size) => {
                session.handle_resized(size);

                let physical = size.to_physical(hidpi_factor);
                swap_chain = r.swap_chain(
                    physical.width as u32,
                    physical.height as u32,
                    present_mode,
                );
                renderer.handle_resized(size, &r);
            }
            platform::WindowEvent::CursorEntered { .. } => {
                win.set_cursor_visible(false);
            }
            platform::WindowEvent::CursorLeft { .. } => {
                win.set_cursor_visible(true);
            }
            platform::WindowEvent::Ready => {
                std::thread::sleep(session.settings.frame_delay);

                let delta = last.elapsed();
                last = time::Instant::now();

                session.frame(&mut session_events, &mut canvas, delta);
                win.request_redraw();

                let pm = session.settings.present_mode();
                if pm != present_mode {
                    present_mode = pm;

                    swap_chain = r.swap_chain(
                        swap_chain.width as u32,
                        swap_chain.height as u32,
                        present_mode,
                    );
                }
            }
            platform::WindowEvent::RedrawRequested => {
                render_timer.run(|avg| {
                    renderer.frame(
                        &session,
                        &avg,
                        &mut r,
                        &mut swap_chain,
                        &canvas,
                    );
                });
                canvas.clear();
            }
            event => {
                session_events.push(event);
            }
        }

        if session.is_running {
            platform::ControlFlow::Continue
        } else {
            platform::ControlFlow::Exit
        }
    });
    Ok(())
}
