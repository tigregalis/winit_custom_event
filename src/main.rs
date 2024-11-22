use std::{num::NonZeroU32, rc::Rc, thread, time::Duration};

use softbuffer::Surface;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

mod winit_app;

fn main() {
    let event_loop = EventLoop::<MyEvent>::with_user_event().build().unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    let app = winit_app::WinitAppBuilder::with_init(move |elwt| {
        let event_loop_proxy = event_loop_proxy.clone();
        thread::spawn(move || loop {
            if event_loop_proxy.send_event(MyEvent).is_err() {
                eprintln!("loop no longer exists");
                break;
            };
            println!("sent event");
            thread::sleep(Duration::from_secs(1));
        });

        let pane = Pane::new(elwt);

        State { pane }
    })
    .with_event_handler(|state, event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        let State { pane } = state;
        let Pane { window, surface } = pane;

        match event {
            Event::UserEvent(MyEvent) => {
                println!("received event");
            }
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::Resized(size) => {
                    if let (Some(x_len), Some(y_len)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    {
                        surface.resize(x_len, y_len).unwrap();
                    }
                }
                WindowEvent::RedrawRequested => {
                    let width = surface.window().inner_size().width;
                    let height = surface.window().inner_size().height;
                    let mut surface_buffer = surface.buffer_mut().unwrap();
                    for (index, pixel) in surface_buffer.iter_mut().enumerate() {
                        let x = index as u32 % width;
                        let y = index as u32 / width;

                        let (r, g, b) = draw(x, y, width, height);

                        let color = u32::from_be_bytes([255, r, g, b]);
                        *pixel = color;
                    }
                    surface_buffer.present().unwrap();
                }
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                _ => {}
            },
            _ => {}
        }
    });
    winit_app::run_app(event_loop, app);
}

struct MyEvent;

struct State {
    pane: Pane,
}

/// A [`winit::window::Window`] paired with a [`softbuffer::Surface`]
struct Pane<D = Rc<Window>, W = D> {
    window: W,
    surface: Surface<D, W>,
}

impl Pane {
    fn new(elwt: &ActiveEventLoop) -> Self {
        let window = winit_app::make_window(elwt, |w| w);
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();
        Self { window, surface }
    }
}

fn draw(x: u32, y: u32, width: u32, height: u32) -> (u8, u8, u8) {
    let x = x as f32;
    let y = y as f32;
    let width = width as f32;
    let height = height as f32;

    const BANDWIDTH: f32 = 20.;
    const R_DEPTH: f32 = 255.;
    const G_DEPTH: f32 = 120.;
    const B_DEPTH: f32 = 160.;
    use core::f32::consts::TAU;

    let r = y / height * R_DEPTH;
    let g = x / width * G_DEPTH;
    let b = (((x * x + y * y).sqrt() * TAU / BANDWIDTH).cos() / 2. + 0.5) * B_DEPTH;

    (r as u8, g as u8, b as u8)
}
