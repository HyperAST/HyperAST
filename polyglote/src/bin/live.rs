use macroquad::{
    miniquad::{
        // conf::{LinuxBackend, Platform},
        // native::{linux_wayland, linux_x11::libx11::Window},
        // start, wayland_interface,
        EventHandler,
    },
    prelude::*,
};
fn window_conf() -> Conf {
    // let mut platform = Platform::default();
    // platform.linux_backend = LinuxBackend::WaylandOnly;
    Conf {
        window_title: "3D".to_owned(),
        high_dpi: true,
        // fullscreen: true,
        sample_count: 4,
        window_height: 2160,
        window_width: 3840,
        window_resizable: true,
        // platform,
        ..Default::default()
    }
}

struct AA;

impl EventHandler for AA {
    fn update(&mut self, _ctx: &mut miniquad::Context) {
        todo!()
    }

    fn draw(&mut self, _ctx: &mut miniquad::Context) {
        todo!()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // start(window_conf(), |&mut ctx| {
    //     Box::new(AA{})
    // });
    dbg!(env!("PWD"));
    println!("Screen {}x{}", screen_width(), screen_height());
    let rust_logo = load_texture("polyglote/examples/rust.png").await.unwrap();
    let ferris = load_texture("polyglote/examples/ferris.png").await.unwrap();

    loop {
        clear_background(LIGHTGRAY);

        // Going 3d!ctx
        // Shader::new(get_context(), r#"

        // "#, fragment_shader, meta);
        // Pipeline::new(ctx, buffer_layout, attributes, shader)
        set_camera(&Camera3D {
            position: vec3(-10., 15., 0.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            viewport: Some((0, 0, 500, 500)),
            // projection: Projection::Orthographics,
            ..Default::default()
        });

        draw_grid(20, 1., BLACK, GRAY);

        draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
        draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
        draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);

        draw_plane(vec3(-8., 0., -8.), vec2(5., 5.), ferris, WHITE);

        draw_cube(vec3(-5., 1., -2.), vec3(2., 2., 2.), rust_logo, WHITE);
        draw_cube(vec3(-5., 1., 2.), vec3(2., 2., 2.), ferris, WHITE);
        draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

        draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);

        set_camera(&Camera3D {
            position: vec3(-20., 15., 0.),
            up: vec3(0., 1., 0.),
            target: vec3(0., 0., 0.),
            viewport: Some((500, 0, 500, 500)),
            ..Default::default()
        });

        draw_grid(20, 1., BLACK, GRAY);

        draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
        draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
        draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);

        draw_plane(vec3(-8., 0., -8.), vec2(5., 5.), ferris, WHITE);

        draw_cube(vec3(-5., 1., -2.), vec3(2., 2., 2.), rust_logo, WHITE);
        draw_cube(vec3(-5., 1., 2.), vec3(2., 2., 2.), ferris, WHITE);
        draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

        draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);

        // Back to screen space, render some text

        set_default_camera();
        draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);

        next_frame().await
    }
}
