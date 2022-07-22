use super::vulkan::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
pub struct App {
    wnd: Window,
}

impl App {
    pub fn create() -> (App, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let wnd = WindowBuilder::new().build(&event_loop).unwrap();
        (App { wnd }, event_loop)
    }

    pub fn run(self, event_loop: EventLoop<()>) -> Option<()> {
        let dims = self.wnd.inner_size();
        let mut playground = VulkanPlayground::create(&self.wnd, dims.width, dims.height)?;
        event_loop.run(move |e, _, control_flow| match e {
            Event::RedrawRequested(window_id) if window_id == self.wnd.id() => {
                playground.render(&self.wnd);
            }
            Event::MainEventsCleared => {
                self.wnd.request_redraw();
            }
            Event::WindowEvent { window_id, event } if window_id == self.wnd.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    playground.resize(size.width, size.height);
                }
                _ => {}
            },
            _ => {}
        });
    }
}
