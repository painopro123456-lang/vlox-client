mod auth;

use macroquad::prelude::*;
use std::sync::mpsc::{self, TryRecvError};
use vlox_client::{cast_block_ray, generate_chunk, generate_world, mine_block, place_block, Block, Inventory, PhysicsBody, Vitals, World, WorldClock};

const BLOCK_SIZE: f32 = 1.0;
const WORLD_SEED: u64 = 0x564C4F585F323032;
const MOUSE_SENSITIVITY: f32 = 0.0025;

struct Player {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    body: PhysicsBody,
    fall_distance: f32,
}

impl Player {
    fn new(spawn_y: f32) -> Self {
        Self {
            position: vec3(0.0, spawn_y + 1.62, 6.0),
            yaw: 0.0,
            pitch: -0.18,
            body: PhysicsBody::new([0.0, spawn_y, 6.0]),
            fall_distance: 0.0,
        }
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ).normalize()
    }

    fn update(&mut self, world: &World, vitals: &mut Vitals, dt: f32) {
        let mut movement = Vec3::ZERO;
        let forward = vec3(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = vec3(self.yaw.cos(), 0.0, self.yaw.sin());

        if is_key_down(KeyCode::W) { movement += forward; }
        if is_key_down(KeyCode::S) { movement -= forward; }
        if is_key_down(KeyCode::D) { movement += right; }
        if is_key_down(KeyCode::A) { movement -= right; }
        let sprinting = is_key_down(KeyCode::LeftControl);
        let was_grounded = self.body.grounded;
        self.body.tick_with_sprint(
            world,
            [movement.x, 0.0, movement.z],
            is_key_pressed(KeyCode::Space),
            sprinting,
            dt,
        );
        if !was_grounded && self.body.grounded {
            vitals.damage((self.fall_distance - 3.0).max(0.0));
            self.fall_distance = 0.0;
        } else if !self.body.grounded && self.body.velocity[1] < 0.0 {
            self.fall_distance += -self.body.velocity[1] * dt;
        }
        vitals.tick(dt, sprinting);
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
        Block::OakLog => Color::new(0.42, 0.27, 0.14, 1.0),
        Block::OakLeaves => Color::new(0.19, 0.48, 0.20, 1.0),
        Block::CoalOre => Color::new(0.24, 0.25, 0.27, 1.0),
        Block::IronOre => Color::new(0.70, 0.49, 0.39, 1.0),
    }
}

fn draw_world(world: &World, center_chunk_x: i32, center_chunk_z: i32) {
    let min_x = (center_chunk_x - 1) * 16;
    let max_x = (center_chunk_x + 2) * 16;
    let min_z = (center_chunk_z - 1) * 16;
    let max_z = (center_chunk_z + 2) * 16;
    for x in min_x..max_x {
        for z in min_z..max_z {
            let top_y = world.surface_height(x, z).unwrap_or(0) as i32;
            for y in 0..=top_y {
                let block = world.get_block(x, y, z);
                if block == Block::Air { continue; }
                let exposed = world.get_block(x + 1, y, z) == Block::Air
                    || world.get_block(x - 1, y, z) == Block::Air
                    || world.get_block(x, y + 1, z) == Block::Air
                    || world.get_block(x, y - 1, z) == Block::Air
                    || world.get_block(x, y, z + 1) == Block::Air
                    || world.get_block(x, y, z - 1) == Block::Air;
                if !exposed { continue; }
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
    let mut world = generate_world(WORLD_SEED, 1);
    let mut inventory = Inventory::default();
    let mut vitals = Vitals::default();
    let mut world_status = String::from("World autosave file: vlox-world.dat");
    let mut mining_target: Option<[i32; 3]> = None;
    let mut mining_progress = 0.0_f32;
    let spawn_y = world.surface_height(0, 6).unwrap_or(24) as f32 + 1.0;
    let mut player = Player::new(spawn_y);
    let mut world_clock = WorldClock::default();
    let mut paused = false;
    let mut frames = 0_u32;
    let mut fps_timer = 0.0_f32;
    let mut shown_fps = 0_u32;
    let (auth_sender, auth_receiver) = mpsc::channel();
    let mut auth_started = false;
    let mut auth_status = String::from("Not signed in");
    let mut auth_code: Option<String> = None;
    let mut auth_name: Option<String> = None;

    loop {
        loop {
            match auth_receiver.try_recv() {
                Ok(auth::AuthEvent::DeviceCode { user_code, verification_uri }) => {
                    auth_code = Some(user_code);
                    auth_status = format!("Open {verification_uri} and enter the code");
                }
                Ok(auth::AuthEvent::SignedIn { name, uuid: _ }) => {
                    auth_name = Some(name.clone());
                    auth_status = format!("Signed in as {name}");
                    auth_code = None;
                }
                Ok(auth::AuthEvent::Failed(error)) => {
                    auth_status = error;
                    auth_code = None;
                    auth_started = false;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    auth_status = String::from("Sign-in worker stopped");
                    auth_started = false;
                    break;
                }
            }
        }
        let login_button = Rect::new(screen_width() - 252.0, 20.0, 232.0, 42.0);
        let (mouse_x, mouse_y) = mouse_position();
        let login_button_clicked = is_mouse_button_pressed(MouseButton::Left)
            && login_button.contains(vec2(mouse_x, mouse_y));
        if login_button_clicked && auth_name.is_none() && !auth_started {
            auth_started = true;
            auth_status = String::from("Connecting to Microsoft...");
            auth_code = None;
            auth::start_sign_in(auth_sender.clone());
        }

        let dt = get_frame_time().min(0.05);
        if is_key_pressed(KeyCode::Escape) {
            paused = !paused;
            if paused { show_mouse(true); } else { show_mouse(false); }
        }

        if !paused && vitals.is_alive() {
            let player_chunk_x = (player.body.position[0].floor() as i32).div_euclid(16);
            let player_chunk_z = (player.body.position[2].floor() as i32).div_euclid(16);
            for chunk_z in player_chunk_z - 1..=player_chunk_z + 1 {
                for chunk_x in player_chunk_x - 1..=player_chunk_x + 1 {
                    if world.chunk(chunk_x, chunk_z).is_none() {
                        world.insert_chunk(chunk_x, chunk_z, generate_chunk(WORLD_SEED, chunk_x, chunk_z));
                    }
                }
            }
        }

        if !vitals.is_alive() && is_key_pressed(KeyCode::R) {
            let respawn_y = world.surface_height(0, 6).unwrap_or(24) as f32 + 1.0;
            player = Player::new(respawn_y);
            vitals = Vitals::default();
            mining_target = None;
            mining_progress = 0.0;
        }

        if !paused {
            world_clock.tick(dt);
            if vitals.is_alive() {
                player.update(&world, &mut vitals, dt);
            }

            if is_key_pressed(KeyCode::Key1) { inventory.select(0); }
            if is_key_pressed(KeyCode::Key2) { inventory.select(1); }
            if is_key_pressed(KeyCode::Key3) { inventory.select(2); }
            if is_key_pressed(KeyCode::Key4) { inventory.select(3); }
            if is_key_pressed(KeyCode::Key5) { inventory.select(4); }
            if is_key_pressed(KeyCode::Key6) { inventory.select(5); }
            if is_key_pressed(KeyCode::Key7) { inventory.select(6); }
            if is_key_pressed(KeyCode::Key8) { inventory.select(7); }
            if is_key_pressed(KeyCode::Key9) { inventory.select(8); }
            if is_key_pressed(KeyCode::F5) {
                let world_result = std::fs::File::create("vlox-world.dat")
                    .and_then(|file| world.save(file));
                let inventory_result = std::fs::File::create("vlox-inventory.dat")
                    .and_then(|file| inventory.save(file));
                world_status = match (world_result, inventory_result) {
                    (Ok(()), Ok(())) => String::from("World and inventory saved"),
                    (Err(error), _) | (_, Err(error)) => format!("Save failed: {error}"),
                };
            }
            if is_key_pressed(KeyCode::F9) {
                let world_result = std::fs::File::open("vlox-world.dat").and_then(World::load);
                let inventory_result = std::fs::File::open("vlox-inventory.dat").and_then(Inventory::load);
                world_status = match (world_result, inventory_result) {
                    (Ok(loaded_world), Ok(loaded_inventory)) => {
                        world = loaded_world;
                        inventory = loaded_inventory;
                        mining_target = None;
                        mining_progress = 0.0;
                        String::from("World and inventory loaded")
                    }
                    (Err(error), _) | (_, Err(error)) => format!("Load failed: {error}"),
                };
            }

            let look = player.forward();
            let origin = [player.position.x, player.position.y, player.position.z];
            let direction = [look.x, look.y, look.z];
            if is_mouse_button_down(MouseButton::Left) && !login_button_clicked {
                if let Some(hit) = cast_block_ray(&world, origin, direction, 5.0) {
                    let block = world.get_block(
                        hit.block_position[0],
                        hit.block_position[1],
                        hit.block_position[2],
                    );
                    if mining_target != Some(hit.block_position) {
                        mining_target = Some(hit.block_position);
                        mining_progress = 0.0;
                    }
                    let required_time = match block {
                        Block::Grass => 0.75,
                        Block::Dirt => 0.75,
                        Block::Sand => 0.40,
                        Block::Stone => 7.50,
                        Block::OakLog => 2.0,
                        Block::OakLeaves => 0.2,
                        Block::CoalOre => 3.0,
                        Block::IronOre => 15.0,
                        Block::Air => 0.0,
                    };
                    mining_progress += dt;
                    if required_time <= 0.0 || mining_progress >= required_time {
                        if let Some(dropped_block) = mine_block(&mut world, hit) {
                            inventory.add(dropped_block);
                        }
                        mining_target = None;
                        mining_progress = 0.0;
                    }
                } else {
                    mining_target = None;
                    mining_progress = 0.0;
                }
            } else {
                mining_target = None;
                mining_progress = 0.0;
            }
            if is_key_pressed(KeyCode::F) {
                if let Some(hit) = cast_block_ray(&world, origin, direction, 5.0) {
                    let feet = player.body.position;
                    if let Some(block) = inventory.selected_block() {
                        if place_block(&mut world, hit, block, feet) {
                            inventory.consume_selected();
                        }
                    }
                }
            }
        }

        let sky = world_clock.sky_rgb();
        clear_background(Color::new(sky[0], sky[1], sky[2], 1.0));

        let camera = Camera3D {
            position: player.position,
            target: player.position + player.forward(),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 70.0,
            projection: Projection::Perspective,
            ..Default::default()
        };
        set_camera(&camera);
        let render_chunk_x = (player.body.position[0].floor() as i32).div_euclid(16);
        let render_chunk_z = (player.body.position[2].floor() as i32).div_euclid(16);
        draw_world(&world, render_chunk_x, render_chunk_z);
        let look = player.forward();
        let origin = [player.position.x, player.position.y, player.position.z];
        let direction = [look.x, look.y, look.z];
        if let Some(hit) = cast_block_ray(&world, origin, direction, 5.0) {
            let [x, y, z] = hit.block_position;
            draw_cube_wires(
                vec3(x as f32, y as f32, z as f32),
                vec3(1.015, 1.015, 1.015),
                Color::new(0.06, 0.06, 0.06, 0.95),
            );
        }
        set_default_camera();

        frames += 1;
        fps_timer += dt;
        if fps_timer >= 0.5 {
            shown_fps = (frames as f32 / fps_timer) as u32;
            frames = 0;
            fps_timer = 0.0;
        }

        draw_rectangle(18.0, 18.0, 650.0, 150.0, Color::new(0.035, 0.055, 0.085, 0.78));
        draw_text("VLOX CLIENT", 32.0, 48.0, 25.0, WHITE);
        draw_text(&format!("FPS  {}", shown_fps), 32.0, 73.0, 18.0, SKYBLUE);
        draw_text(&format!("HP {:02}   FOOD {:02}", vitals.health.round() as u32, vitals.food.round() as u32), 155.0, 73.0, 17.0, WHITE);
        draw_text("WASD move | CTRL sprint | SPACE jump | RMB look | LMB mine | F place | 1-9 hotbar", 32.0, 98.0, 13.0, LIGHTGRAY);
        draw_text("F5 save | F9 load (world + inventory)", 32.0, 130.0, 15.0, LIGHTGRAY);
        draw_text(&world_status, 32.0, 154.0, 16.0, LIGHTGRAY);

        draw_rectangle(login_button.x, login_button.y, login_button.w, login_button.h,
            Color::new(0.08, 0.18, 0.30, 0.94));
        draw_rectangle_lines(login_button.x, login_button.y, login_button.w, login_button.h, 1.5, SKYBLUE);
        let login_label = if let Some(name) = &auth_name {
            format!("Microsoft: {name}")
        } else if auth_started {
            String::from("Microsoft sign-in...")
        } else {
            String::from("Sign in with Microsoft")
        };
        draw_text(&login_label, login_button.x + 12.0, login_button.y + 27.0, 17.0, WHITE);
        if auth_started || auth_name.is_some() {
            let panel_y = 70.0;
            draw_rectangle(login_button.x - 130.0, panel_y, 362.0, 92.0,
                Color::new(0.025, 0.04, 0.065, 0.94));
            draw_text(&auth_status, login_button.x - 118.0, panel_y + 24.0, 13.0, LIGHTGRAY);
            if let Some(code) = &auth_code {
                draw_text(&format!("Code: {code}"), login_button.x - 118.0, panel_y + 53.0, 23.0, YELLOW);
                draw_text("Complete sign-in in your browser", login_button.x - 118.0, panel_y + 76.0, 12.0, WHITE);
            }
        }

        if mining_target.is_some() {
            let block = mining_target.map(|p| world.get_block(p[0], p[1], p[2])).unwrap_or(Block::Air);
            let required = match block {
                Block::Grass | Block::Dirt => 0.75,
                Block::Sand => 0.40,
                Block::Stone => 7.50,
                Block::OakLog => 2.0,
                Block::OakLeaves => 0.2,
                Block::CoalOre => 3.0,
                Block::IronOre => 15.0,
                Block::Air => 1.0,
            };
            let progress = (mining_progress / required).clamp(0.0, 1.0);
            let bar_x = screen_width() * 0.5 - 90.0;
            let bar_y = screen_height() - 92.0;
            draw_rectangle(bar_x, bar_y, 180.0, 8.0, Color::new(0.02, 0.03, 0.04, 0.85));
            draw_rectangle(bar_x, bar_y, 180.0 * progress, 8.0, Color::new(0.92, 0.82, 0.36, 1.0));
        }

        let hotbar_width = 9.0 * 54.0 + 8.0 * 8.0;
        let hotbar_x = (screen_width() - hotbar_width) * 0.5;
        let hotbar_y = screen_height() - 76.0;
        for slot in 0..9 {
            let x = hotbar_x + slot as f32 * 62.0;
            draw_rectangle(x, hotbar_y, 54.0, 54.0, Color::new(0.035, 0.045, 0.06, 0.82));
            if let Some(stack) = inventory.stack(slot) {
                if let Some(block) = stack.block {
                    if stack.count > 0 {
                        draw_rectangle(x + 13.0, hotbar_y + 10.0, 28.0, 28.0, block_color(block));
                        draw_text(&stack.count.to_string(), x + 34.0, hotbar_y + 50.0, 14.0, WHITE);
                    }
                }
            }
            if slot == inventory.selected_slot() {
                draw_rectangle_lines(x - 2.0, hotbar_y - 2.0, 58.0, 58.0, 3.0, YELLOW);
            }
            draw_text(&(slot + 1).to_string(), x + 4.0, hotbar_y + 49.0, 12.0, WHITE);
        }

        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.5;
        draw_line(cx - 7.0, cy, cx + 7.0, cy, 2.0, WHITE);
        draw_line(cx, cy - 7.0, cx, cy + 7.0, 2.0, WHITE);

        if !vitals.is_alive() {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.25, 0.01, 0.01, 0.72));
            let title = "YOU DIED";
            let width = measure_text(title, None, 48, 1.0).width;
            draw_text(title, (screen_width() - width) * 0.5, screen_height() * 0.5, 48.0, WHITE);
            draw_text("Press R to respawn", screen_width() * 0.5 - 95.0, screen_height() * 0.5 + 42.0, 22.0, WHITE);
        }

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
