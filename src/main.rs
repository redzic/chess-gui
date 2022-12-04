use sdl2::event::Event;
use sdl2::hint::{self, Hint};
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::video::{gl_attr, GLProfile};
use std::time::Duration;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

pub fn main() {
    // let sdl_context = sdl2::init().unwrap();

    // // hint::set_with_priority("SDL_RENDER_SCALE_QUALITY", "1", &Hint::Override);

    // let video_subsystem = sdl_context.video().unwrap();

    // let window = video_subsystem
    //     .window("Chess AI", 8 * 80, 8 * 80)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();

    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem
    //     .window("Chess AI", 8 * 80, 8 * 80)
    //     .opengl() // this line DOES NOT enable opengl, but allows you to create/get an OpenGL context from your window.
    //     .build()
    //     .unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    // Don't use deprecated OpenGL functions
    gl_attr.set_context_profile(GLProfile::Core);

    // Set the context into debug mode
    gl_attr.set_context_flags().debug().set();

    // Set the OpenGL context version (OpenGL 3.2)
    gl_attr.set_context_version(3, 2);

    // Enable anti-aliasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 8 * 80, 8 * 80)
        .opengl()
        .build()
        .unwrap();

    // Yes, we're still using the Core profile
    assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    // ... and we're still using OpenGL 3.2
    assert_eq!(gl_attr.context_version(), (3, 2));

    dbg!(hint::set("SDL_RENDER_SCALE_QUALITY", "1"));
    dbg!(hint::get("SDL_RENDER_SCALE_QUALITY"));

    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();

    let tc = canvas.texture_creator();

    let t = tc.load_texture("./white_pawn.png").unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(132, 134, 232));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(30, 31, 79));
        for i in 0..8 {
            for j in 0..8 {
                if (i & 1 == 0) ^ (j & 1 == 0) {
                    canvas.fill_rect(Rect::new(80 * i, 80 * j, 80, 80)).unwrap();
                }
            }
        }

        // canvas.copy(&t, None, None).unwrap();

        // canvas.copy(&t, None, Rect::new(0, 0, 80, 80)).unwrap();
        canvas
            .copy(&t, None, Rect::new(0, 0, 80 * 8, 80 * 8))
            .unwrap();

        // canvas.

        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
