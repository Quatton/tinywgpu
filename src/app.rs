#![allow(dead_code)]

use std::fmt::Debug;

use crate::graphics::{create_graphics, Graphics, Rc};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

enum State {
    Ready(Graphics),
    Init(Option<EventLoopProxy<Graphics>>),
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Ready(_) => write!(f, "Ready"),
            State::Init(_) => write!(f, "Init"),
        }
    }
}

pub struct App {
    name: String,
    state: State,
}

impl App {
    pub fn new() -> Self {
        Self {
            name: std::env!("CARGO_PKG_NAME").to_string(),
            state: State::Init(None),
        }
    }

    pub fn configure(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_event_loop(mut self, event_loop: &EventLoop<Graphics>) -> Self {
        self.state = State::Init(Some(event_loop.create_proxy()));
        self
    }

    fn draw(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.draw();
        } else {
            log::warn!("Attempted to draw before graphics were ready.");
        }
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.resize(size);
        }
    }
}

impl ApplicationHandler<Graphics> for App {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let State::Init(proxy) = &mut self.state {
            if let Some(proxy) = proxy.take() {
                let mut win_attr = Window::default_attributes();

                #[cfg(not(target_arch = "wasm32"))]
                {
                    win_attr = win_attr.with_title(self.name.as_str());
                }

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    win_attr = win_attr.with_append(true);
                }

                let window = Rc::new(
                    event_loop
                        .create_window(win_attr)
                        .expect("create window err."),
                );

                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(create_graphics(window, proxy));

                #[cfg(not(target_arch = "wasm32"))]
                pollster::block_on(create_graphics(window, proxy));
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut graphics: Graphics) {
        graphics.draw();
        log::info!("User event received.");
        self.state = State::Ready(graphics);
    }
}
