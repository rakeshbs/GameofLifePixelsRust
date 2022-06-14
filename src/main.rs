#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

struct GameOfLife {
    state: Vec<bool>,
}

fn main() -> Result<(), Error> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .unwrap();

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Game of Life")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut game_of_life = GameOfLife::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            game_of_life.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            game_of_life.update();
            window.request_redraw();
            std::thread::sleep(std::time::Duration::from_millis(60));
        }
    });
}

impl GameOfLife {
    fn new() -> Self {
        let mut state = vec![false; (WIDTH * HEIGHT) as usize];
        let mut rng = rand::thread_rng();

        for i in 0..state.len() {
            let n = rng.gen_range(0..10);
            if n >= 8 {
                state[i] = true;
            }
        }
        GameOfLife { state }
    }

    #[inline]
    fn count_neighbours(&self, idx: usize) -> u32 {
        let mut count = 0;
        for i in [-1, 0, 1] as [i32; 3] {
            for j in [-1, 0, 1] as [i32; 3] {
                let neighbour_idx = idx as i32 + (WIDTH as i32) * i + j;
                if neighbour_idx >= 0 && neighbour_idx < (WIDTH * HEIGHT) as i32 {
                    if self.state[neighbour_idx as usize] {
                        count += 1;
                    }
                }
            }
        }
        return count;
    }

    fn update(&mut self) {
        self.state = self
            .state
            .iter()
            .enumerate()
            .map(|(i, &alive)| {
                let neighbour_count = self.count_neighbours(i);
                if alive {
                    if neighbour_count == 2 || neighbour_count == 3 {
                        return true;
                    }
                } else {
                    if neighbour_count == 3 {
                        return true;
                    }
                }
                return false;
            })
            .collect::<Vec<bool>>()
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = if self.state[i] {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
