use macroquad::prelude::*;
use vlox_client::{cast_block_ray, Block, PhysicsBody, World};

const WORLD_RADIUS: i32 = 12;
const BLOCK_SIZE: f32 = 1.0;
const MOUSE_SENSITIVITY: f32 = 0.0025;

struct Player {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    body: PhysicsBody,
}

impl Player {
    fn new() -> Self {
        Self {
            position: vec3(0.0, 5.62, 6.0),
            yaw: 0.0,
            pitch: -0.18,
            body: PhysicsBody::new([0.0, 4.0, 6.0]),
        }
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ).normalize()
    }

    fn update(&mut self, world: &World, dt: f32) {
        let mut movement = Vec3::ZERO;
        let forward = vec3(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = vec3(self.yaw.cos(), 0.0, self.yaw.sin());

        if is_key_down(KeyCode::W) { movement += forward; }
        if is_key_down(KeyCode::S) { movement -= forward; }
        if is_key_down(KeyCode::D) { movement += right; }
        if is_key_down(KeyCode::A) { movement -= right; }
        self.body.tick(
            world,
            [movement.x, 0.0, movement.z],
            is_key_pressed(KeyCode::Space),
            dt,
        );
        self.position = vec3(
            self.body.position[0],
            self.body.position[1] + 1.62,
            self.body.position[2],
        );

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

fn block_color(block: Block) -> Color {
    match block {
        Block::Air => BLANK,
        Block::Grass => Color::new(0.30, 0.66, 0.28, 1.0),
        Block::Dirt => Color::new(0.48, 0.31, 0.18, 1.0),
        Block::Stone => Color::new(0.48, 0.49, 0.51, 1.0),
        Block::Sand => Color::new(0.83, 0.76, 0.52, 1.0),
    }
}

fn draw_world(world: &World) {
    for x in 0..16_i32 {
        for z in 0..16_i32 {
            for y in 0..64_i32 {
                let block = world.get_block(x, y, z);
                if block == Block::Air { continue; }
                draw_cube(
                    vec3(x as f32, y as f32, z as f32),
                    vec3(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
                    None,
                    block_color(block),
                );
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new_flat_test_world();
    let hotbar = [Block::Grass, Block::Dirt, Block::Stone, Block::Sand];
    let mut selected_slot = 0_usize;
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
            player.update(&world, dt);

            if is_key_pressed(KeyCode::Key1) { selected_slot = 0; }
            if is_key_pressed(KeyCode::Key2) { selected_slot = 1; }
            if is_key_pressed(KeyCode::Key3) { selected_slot = 2; }
            if is_key_pressed(KeyCode::Key4) { selected_slot = 3; }

            let look = player.forward();
            let origin = [player.position.x, player.position.y, player.position.z];
            let direction = [look.x, look.y, look.z];
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(hit) = cast_block_ray(&world, origin, direction, 5.0) {
                    world.set_block(hit.block_position[0], hit.block_position[1], hit.block_position[2], Block::Air);
                }
            }
            if is_key_pressed(KeyCode::F) {
                if let Some(hit) = cast_block_ray(&world, origin, direction, 5.0) {
                    let [x, y, z] = hit.adjacent_position;
                    let px = player.body.position[0];
                    let py = player.body.position[1];
                    let pz = player.body.position[2];
                    let overlaps_player = x as f32 + 1.0 > px - 0.3
                        && (x as f32) < px + 0.3
                        && y as f32 + 1.0 > py
                        && (y as f32) < py + 1.8
                        && z as f32 + 1.0 > pz - 0.3
                        && (z as f32) < pz + 0.3;
                    if !overlaps_player && world.get_block(x, y, z) == Block::Air {
                        world.set_block(x, y, z, hotbar[selected_slot]);
                    }
                }
            }
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
        draw_world(&world);
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
        draw_text("WASD move | SPACE jump | RMB look | LMB mine | F place", 32.0, 98.0, 15.0, LIGHTGRAY);

        let hotbar_width = 4.0 * 54.0 + 3.0 * 8.0;
        let hotbar_x = (screen_width() - hotbar_width) * 0.5;
        let hotbar_y = screen_height() - 76.0;
        for (slot, block) in hotbar.iter().enumerate() {
            let x = hotbar_x + slot as f32 * 62.0;
            draw_rectangle(x, hotbar_y, 54.0, 54.0, Color::new(0.035, 0.045, 0.06, 0.82));
            draw_rectangle(x + 13.0, hotbar_y + 12.0, 28.0, 28.0, block_color(*block));
            if slot == selected_slot {
                draw_rectangle_lines(x - 2.0, hotbar_y - 2.0, 58.0, 58.0, 3.0, YELLOW);
            }
            draw_text(&(slot + 1).to_string(), x + 4.0, hotbar_y + 49.0, 12.0, WHITE);
        }

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
