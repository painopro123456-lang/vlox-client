use macroquad::prelude::*;

const WORLD_RADIUS: i32 = 12;
const BLOCK_SIZE: f32 = 1.0;
const PLAYER_SPEED: f32 = 6.0;
const MOUSE_SENSITIVITY: f32 = 0.0025;

struct Player {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            position: vec3(0.0, 2.0, 6.0),
            yaw: 0.0,
            pitch: -0.18,
        }
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ).normalize()
    }

    fn update(&mut self, dt: f32) {
        let mut movement = Vec3::ZERO;
        let forward = vec3(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = vec3(self.yaw.cos(), 0.0, self.yaw.sin());

        if is_key_down(KeyCode::W) { movement += forward; }
        if is_key_down(KeyCode::S) { movement -= forward; }
        if is_key_down(KeyCode::D) { movement += right; }
        if is_key_down(KeyCode::A) { movement -= right; }
        if is_key_down(KeyCode::Space) { self.position.y += PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::LeftShift) { self.position.y -= PLAYER_SPEED * dt; }

        if movement.length_squared() > 0.0 {
            self.position += movement.normalize() * PLAYER_SPEED * dt;
        }

        if is_mouse_button_down(MouseButton::Right) {
            let delta = mouse_delta_position();
            self.yaw += delta.x * MOUSE_SENSITIVITY * 100.0;
            self.pitch = (self.pitch - delta.y * MOUSE_SENSITIVITY * 100.0).clamp(-1.45, 1.45);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "VLOX CLIENT".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

fn draw_world() {
    // A small procedural block world: no proprietary game assets are bundled.
    for x in -WORLD_RADIUS..=WORLD_RADIUS {
        for z in -WORLD_RADIUS..=WORLD_RADIUS {
            let edge = x.abs() == WORLD_RADIUS || z.abs() == WORLD_RADIUS;
            let tint = if edge {
                Color::new(0.19, 0.43, 0.28, 1.0)
            } else if (x + z) % 5 == 0 {
                Color::new(0.30, 0.62, 0.34, 1.0)
            } else {
                Color::new(0.25, 0.54, 0.30, 1.0)
            };
            draw_cube(
                vec3(x as f32 * BLOCK_SIZE, -0.5, z as f32 * BLOCK_SIZE),
                vec3(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
                None,
                Some(tint),
            );
        }
    }

    // A few raised blocks make the prototype world visibly voxel-based.
    for (x, y, z, color) in [
        (2.0, 0.0, 1.0, Color::new(0.48, 0.48, 0.52, 1.0)),
        (2.0, 1.0, 1.0, Color::new(0.48, 0.48, 0.52, 1.0)),
        (3.0, 0.0, 1.0, Color::new(0.48, 0.48, 0.52, 1.0)),
        (-3.0, 0.0, -2.0, Color::new(0.55, 0.38, 0.22, 1.0)),
        (-3.0, 1.0, -2.0, Color::new(0.55, 0.38, 0.22, 1.0)),
    ] {
        draw_cube(vec3(x, y, z), vec3(1.0, 1.0, 1.0), None, Some(color));
        draw_cube_wires(vec3(x, y, z), vec3(1.0, 1.0, 1.0), BLACK);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player::new();
    let mut paused = false;
    let mut frames = 0_u32;
    let mut fps_timer = 0.0_f32;
    let mut shown_fps = 0_u32;

    loop {
        let dt = get_frame_time().min(0.05);
        if is_key_pressed(KeyCode::Escape) {
            paused = !paused;
            if paused { show_mouse(true); } else { show_mouse(false); }
        }

        if !paused {
            player.update(dt);
        }

        clear_background(Color::new(0.48, 0.70, 0.91, 1.0));

        let camera = Camera3D {
            position: player.position,
            target: player.position + player.forward(),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 70.0,
            projection: Projection::Perspective,
            ..Default::default()
        };
        set_camera(&camera);
        draw_world();
        set_default_camera();

        frames += 1;
        fps_timer += dt;
        if fps_timer >= 0.5 {
            shown_fps = (frames as f32 / fps_timer) as u32;
            frames = 0;
            fps_timer = 0.0;
        }

        draw_rectangle(18.0, 18.0, 235.0, 92.0, Color::new(0.035, 0.055, 0.085, 0.78));
        draw_text("VLOX CLIENT", 32.0, 48.0, 25.0, WHITE);
        draw_text(&format!("FPS  {}", shown_fps), 32.0, 73.0, 18.0, SKYBLUE);
        draw_text("WASD move  |  SPACE/SHIFT fly  |  RMB look", 32.0, 98.0, 15.0, LIGHTGRAY);

        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.5;
        draw_line(cx - 7.0, cy, cx + 7.0, cy, 2.0, WHITE);
        draw_line(cx, cy - 7.0, cx, cy + 7.0, 2.0, WHITE);

        if paused {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.01, 0.02, 0.04, 0.72));
            let title = "PAUSED";
            let width = measure_text(title, None, 48, 1.0).width;
            draw_text(title, (screen_width() - width) * 0.5, screen_height() * 0.5, 48.0, WHITE);
            let hint = "Press ESC to return";
            let hint_width = measure_text(hint, None, 22, 1.0).width;
            draw_text(hint, (screen_width() - hint_width) * 0.5, screen_height() * 0.5 + 42.0, 22.0, LIGHTGRAY);
        }

        next_frame().await;
    }
}
