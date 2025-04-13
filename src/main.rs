use macroquad::prelude::*;
use macroquad::window::Conf;
use rand::gen_range;
use std::{cmp, collections::HashSet, vec};

fn draw_text_h_centered(text: &str, y: f32, font_size: u16) {
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    let x = (screen_width() - text_dimensions.width) / 2.0;
    draw_text(text, x, y, font_size as f32, WHITE);
}

fn distance(p1: &Vec2, p2: &Vec2) -> f32 {
    ((p2.x - p1.x).powf(2.0) + (p2.y - p1.y).powf(2.0)).sqrt()
}

struct Ship {
    position: Vec2,
    velocity: Vec2,
    health: usize,
    iframes: u32,
    // Rotation in radians
    rotation: f32,
}
impl Ship {
    fn new(x: f32, y: f32) -> Ship {
        let rotation_degrees: f32 = 270.0;
        Ship {
            position: Vec2::new(x, y),
            velocity: Vec2::new(0.0, 0.0),
            health: 5,
            iframes: 120,
            rotation: rotation_degrees.to_radians(),
        }
    }

    fn render(&self) {
        let vertices = self.vertices();
        if self.health > 0 {
            draw_triangle_lines(vertices[0], vertices[1], vertices[2], 1.0, WHITE)
        }
    }

    fn take_hit(&mut self) {
        if self.iframes == 0 && self.health > 0 {
            self.health -= 1;
            self.iframes = 30;
        }
    }

    fn vertices(&self) -> Vec<Vec2> {
        let x1 = self.position.x;
        let y1 = self.position.y;
        let x2 = self.position.x + 45.0;
        let y2 = self.position.y - 15.0;
        let x3 = self.position.x;
        let y3 = self.position.y - 30.0;

        let center = Vec2::new((x1 + x2 + x3) / 3.0, (y1 + y2 + y3) / 3.0);

        vec![Vec2::new(x1, y1), Vec2::new(x2, y2), Vec2::new(x3, y3)]
            .iter()
            .map(|&vertex| {
                // translate the point so it's relative to the origin
                let x = vertex.x - center.x;
                let y = vertex.y - center.y;
                // apply rotation matrix
                let rotated = Vec2::new(
                    x * self.rotation.cos() - y * self.rotation.sin(),
                    x * self.rotation.sin() + y * self.rotation.cos(),
                );
                // translate back to original location
                rotated + center
            })
            .collect()
    }
}

#[derive(Clone)]
struct Laser {
    id: u32,
    position: Vec2,
    velocity: Vec2,
}
impl Laser {
    fn new(x_pos: f32, y_pos: f32, x_vel: f32, y_vel: f32, id: u32) -> Laser {
        Laser {
            id,
            position: Vec2::new(x_pos, y_pos),
            velocity: Vec2::new(x_vel, y_vel),
        }
    }

    fn render(&self) {
        let length = 10.0;
        let angle = self.velocity.y.atan2(self.velocity.x);
        draw_line(
            self.position.x,
            self.position.y,
            self.position.x + length * angle.cos(),
            self.position.y + length * angle.sin(),
            1.0,
            WHITE,
        )
    }

    fn tick(&mut self, frame_time: f32) {
        self.position.x += self.velocity.x * frame_time;
        self.position.y += self.velocity.y * frame_time;
    }
}

#[derive(Clone)]
struct Asteroid {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    rotation: f32,
    health: u32,
    num_sides: u8,
}
impl Asteroid {
    fn new(x_pos: f32, y_pos: f32, x_vel: f32, y_vel: f32, radius: f32, id: u32) -> Asteroid {
        Asteroid {
            id,
            position: Vec2::new(x_pos, y_pos),
            velocity: Vec2::new(x_vel, y_vel),
            radius,
            rotation: 0.0,
            health: 1,
            num_sides: 8,
        }
    }

    fn render(&self) {
        draw_poly_lines(
            self.position.x,
            self.position.y,
            self.num_sides,
            self.radius,
            self.rotation,
            1.0,
            WHITE,
        );
    }

    fn tick(&mut self, frame_time: f32) {
        self.position.x += self.velocity.x * frame_time;
        self.position.y += self.velocity.y * frame_time;
        self.rotation += 30.0 * frame_time;
    }

    fn take_hit(&mut self) {
        if self.health > 0 {
            self.health -= 1;
        }
    }
}

struct Game {
    width: f32,
    height: f32,
    center: Vec2,
    player: Ship,
    asteroids: Vec<Asteroid>,
    asteroid_counter: u32,
    max_asteroids: usize,
    lasers: Vec<Laser>,
    laser_counter: u32,
    laser_cooldown: f32,
    laser_cooldown_remaining: f32,
    score: u32,
}
impl Game {
    fn new() -> Game {
        let width = screen_width();
        let height = screen_height();
        let center = Vec2::new(width / 2.0, height / 2.0);

        let mut game = Game {
            width,
            height,
            center,
            player: Ship::new(center.x, center.y),
            asteroids: vec![],
            asteroid_counter: 0,
            max_asteroids: 20,
            lasers: vec![],
            laser_counter: 0,
            laser_cooldown: 0.2,
            laser_cooldown_remaining: 0.0,
            score: 0,
        };
        game.generate_asteroids();
        game
    }

    fn reset(&mut self) {
        let width = screen_width();
        let height = screen_height();
        let center = Vec2::new(width / 2.0, height / 2.0);

        self.asteroids = vec![];
        self.generate_asteroids();
        self.lasers = vec![];
        self.player = Ship::new(center.x, center.y);
        self.score = 0;
    }

    fn render(&self) {
        draw_text(&format!("Score: {}", self.score), 10.0, 28.0, 28.0, WHITE);
        draw_text(
            &format!("Health: {}", "<3 ".repeat(self.player.health)),
            150.0,
            28.0,
            28.0,
            WHITE,
        );

        self.player.render();

        for a in &self.asteroids {
            a.render();
        }
        for l in &self.lasers {
            l.render();
        }
    }

    fn tick(&mut self, frame_time: f32) {
        let rotation_degrees: f32 = 250.0 * frame_time;
        let thrust: f32 = 5.0 * frame_time;

        // Check for movement input
        if is_key_down(KeyCode::W) {
            // Apply forward thrust
            self.player.velocity.x += thrust * self.player.rotation.cos();
            self.player.velocity.y += thrust * self.player.rotation.sin();
        } else if is_key_down(KeyCode::S) {
            // Apply backward thrust
            self.player.velocity.x -= thrust * self.player.rotation.cos();
            self.player.velocity.y -= thrust * self.player.rotation.sin();
        }

        if is_key_down(KeyCode::A) {
            // Rotate left
            self.player.rotation -= rotation_degrees.to_radians();
        } else if is_key_down(KeyCode::D) {
            // Rotate right
            self.player.rotation += rotation_degrees.to_radians();
        }

        // Update player position according to velocity
        let min_x: f32 = 0.0;
        let max_x: f32 = self.width;
        self.player.position.x =
            max_x.min(min_x.max(self.player.position.x + self.player.velocity.x));

        let min_y: f32 = 0.0;
        let max_y: f32 = self.height;
        self.player.position.y =
            max_y.min(min_y.max(self.player.position.y + self.player.velocity.y));

        if self.player.position.x == min_x || self.player.position.x == max_x {
            self.player.velocity.x = 0.0;
        }

        if self.player.position.y == min_y || self.player.position.y == max_y {
            self.player.velocity.y = 0.0;
        }

        // Check for firing
        const LAZER_VEL: f32 = 400.00;
        if self.laser_cooldown_remaining <= 0.0 && is_key_down(KeyCode::Space) {
            self.laser_counter += 1;
            let front = self.player.vertices()[1];
            let fired_laser = Laser::new(
                front.x,
                front.y,
                self.player.velocity.x + LAZER_VEL * self.player.rotation.cos(),
                self.player.velocity.y + LAZER_VEL * self.player.rotation.sin(),
                self.laser_counter,
            );
            self.lasers.push(fired_laser);
            self.laser_cooldown_remaining = self.laser_cooldown;
        }

        if self.laser_cooldown_remaining > 0.0 {
            self.laser_cooldown_remaining -= frame_time;
        }

        if self.player.iframes > 0 {
            self.player.iframes -= 1;
        }

        let mut remove_asteroid_ids: HashSet<u32> = HashSet::new();
        for a in self.asteroids.iter_mut() {
            a.tick(frame_time);

            // destroy offscreen asteroids
            if a.position.x > self.width + a.radius
                || a.position.y > self.height + a.radius
                || a.position.x < -a.radius
                || a.position.y < -a.radius
            {
                remove_asteroid_ids.insert(a.id);
            }

            // check for collision with player
            for p in self.player.vertices() {
                if distance(&p, &a.position) < a.radius {
                    self.player.take_hit();
                    remove_asteroid_ids.insert(a.id);
                }
            }
        }

        // check for lasers hitting asteroids
        let mut remove_laser_ids: HashSet<u32> = HashSet::new();
        let mut split_asteroids: Vec<Asteroid> = vec![];
        for l in self.lasers.iter_mut() {
            l.tick(frame_time);

            // check for contact with an asteroid
            for a in self.asteroids.iter_mut() {
                if distance(&l.position, &a.position) < a.radius {
                    a.take_hit();
                    remove_laser_ids.insert(l.id);
                    if a.health == 0 {
                        remove_asteroid_ids.insert(a.id);

                        // Split asteroid
                        if a.radius > 20.0 {
                            let new_radius = a.radius / 2.0;
                            split_asteroids.push(Asteroid::new(
                                a.position.x,
                                a.position.y,
                                -(a.velocity.y / 2.0),
                                a.velocity.y,
                                new_radius,
                                self.asteroid_counter + 1,
                            ));
                            split_asteroids.push(Asteroid::new(
                                a.position.x,
                                a.position.y,
                                a.velocity.y / 2.0,
                                a.velocity.y,
                                new_radius,
                                self.asteroid_counter + 2,
                            ));
                            self.asteroid_counter += 2;
                        }

                        self.score += 1;
                    }
                    break;
                }
            }

            // check for offscreen lasers
            if l.position.x > self.width
                || l.position.y > self.height
                || l.position.x < 0.0
                || l.position.y < 0.0
            {
                remove_laser_ids.insert(l.id);
            }
        }

        self.asteroids = self
            .asteroids
            .iter()
            .cloned()
            .filter(|a| !remove_asteroid_ids.contains(&a.id))
            .collect();

        self.lasers = self
            .lasers
            .iter()
            .cloned()
            .filter(|l| !remove_laser_ids.contains(&l.id))
            .collect();

        self.generate_asteroids();

        self.asteroids.extend(split_asteroids);
    }

    fn generate_asteroids(&mut self) {
        // Split generation evenly across the 4 screen boundaries
        // Generate asteroids moving roughly toward the center of the screen

        let num_asteroids = self.max_asteroids - cmp::min(self.asteroids.len(), self.max_asteroids);
        let asteroids_per_boundary = num_asteroids / 4;

        let min_radius = 10.0;
        let max_radius = 100.0;
        let speed = 100.0;
        let angle_variation_degrees = 30.0;

        // Left boundary
        for _ in 0..asteroids_per_boundary {
            let radius: f32 = gen_range(min_radius, max_radius);
            let y: f32 = gen_range(radius, self.height - radius);

            let delta_x = self.center.x;
            let delta_y = self.center.y - y;

            let angle_toward_center = delta_y.atan2(delta_x).to_degrees();

            // add random variation to the angle
            let angle =
                (angle_toward_center + gen_range(0.0, angle_variation_degrees)).to_radians();
            let x_vel = speed * angle.cos();
            let y_vel = speed * angle.sin();

            self.asteroid_counter += 1;
            self.asteroids.push(Asteroid::new(
                0.0,
                y,
                x_vel,
                y_vel,
                radius,
                self.asteroid_counter,
            ))
        }

        // Top boundary
        for _ in 0..asteroids_per_boundary {
            let radius: f32 = gen_range(min_radius, max_radius);
            let x: f32 = gen_range(radius, self.width - radius);
            let delta_x = self.center.x - x;
            let delta_y = self.center.y;

            let angle_toward_center = delta_y.atan2(delta_x).to_degrees();

            // add random variation to the angle
            let angle =
                (angle_toward_center + gen_range(0.0, angle_variation_degrees)).to_radians();
            let x_vel = speed * angle.cos();
            let y_vel = speed * angle.sin();

            self.asteroid_counter += 1;
            self.asteroids.push(Asteroid::new(
                x,
                0.0,
                x_vel,
                y_vel,
                radius,
                self.asteroid_counter,
            ))
        }

        // Right boundary
        for _ in 0..asteroids_per_boundary {
            let radius: f32 = gen_range(min_radius, max_radius);
            let y: f32 = gen_range(radius, self.height - radius);
            let delta_x = self.center.x - self.width;
            let delta_y = self.center.y - y;

            let angle_toward_center = delta_y.atan2(delta_x).to_degrees();

            // add random variation to the angle
            let angle =
                (angle_toward_center + gen_range(0.0, angle_variation_degrees)).to_radians();
            let x_vel = speed * angle.cos();
            let y_vel = speed * angle.sin();

            self.asteroid_counter += 1;
            self.asteroids.push(Asteroid::new(
                self.width,
                y,
                x_vel,
                y_vel,
                radius,
                self.asteroid_counter,
            ))
        }

        // Bottom boundary
        for _ in 0..asteroids_per_boundary {
            let radius: f32 = gen_range(min_radius, max_radius);
            let x: f32 = gen_range(radius, self.width - radius);
            let delta_x = self.center.x - x;
            let delta_y = self.center.y - self.height;

            let angle_toward_center = delta_y.atan2(delta_x).to_degrees();

            // add random variation to the angle
            let angle =
                (angle_toward_center + gen_range(0.0, angle_variation_degrees)).to_radians();
            let x_vel = speed * angle.cos();
            let y_vel = speed * angle.sin();

            self.asteroid_counter += 1;
            self.asteroids.push(Asteroid::new(
                x,
                self.height,
                x_vel,
                y_vel,
                radius,
                self.asteroid_counter,
            ))
        }
    }

    fn check_game_over(&self) -> bool {
        if self.player.health == 0 {
            draw_text_h_centered("Game Over", self.center.y, 48);
            draw_text_h_centered(&format!("Score: {}", self.score), self.center.y + 50.0, 28);
            draw_text_h_centered("Press enter to play again", self.center.y + 100.0, 28);
            return true;
        } else if self.score == 100 {
            draw_text_h_centered("You Win", self.center.y, 48);
            draw_text_h_centered(&format!("Score: {}", self.score), self.center.y + 50.0, 28);
            draw_text_h_centered("Press enter to play again", self.center.y + 100.0, 28);
            return true;
        }
        false
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Asteroids"),
        window_resizable: false,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    let mut game_started = false;
    let mut game_over = false;

    loop {
        let frame_time: f32 = get_frame_time();

        clear_background(BLACK);
        if !game_started {
            draw_text_h_centered("Asteroids", game.center.y, 50);
            draw_text_h_centered("Press enter to start the game", game.center.y + 50.0, 28);
        }

        if !game_over && game_started {
            game.tick(frame_time);
            game.render();
        } else if is_key_down(KeyCode::Enter) {
            game.reset();
            game_over = false;
            game_started = true;
            continue;
        }
        game_over = game.check_game_over();

        next_frame().await
    }
}
