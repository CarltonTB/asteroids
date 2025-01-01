use std::{collections::HashSet, vec};

use macroquad::prelude::*;
use rand::gen_range;

fn draw_text_h_centered(text: &str, y: f32, font_size: u16) {
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    let x = (screen_width() - text_dimensions.width) / 2.0;
    draw_text(text, x, y, font_size as f32, WHITE);
}

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}
impl Position {
    fn new(x: f32, y: f32) -> Position {
        Position { x, y }
    }

    fn distance_to(&self, p: &Position) -> f32 {
        ((p.x - self.x).powf(2.0) + (p.y - self.y).powf(2.0)).sqrt()
    }
}

#[derive(Clone)]
struct Velocity {
    // Velocity in pixels per second
    x: f32,
    y: f32,
}
impl Velocity {
    fn new(x: f32, y: f32) -> Velocity {
        Velocity { x, y }
    }
}

struct Ship {
    position: Position,
    health: usize,
    iframes: u32,
    // rotation: f32,
}
impl Ship {
    fn new(x: f32, y: f32) -> Ship {
        Ship {
            position: Position::new(x, y),
            health: 5,
            iframes: 120,
        }
    }

    fn v1(&self) -> Vec2 {
        Vec2::new(self.position.x, self.position.y)
    }

    fn v2(&self) -> Vec2 {
        Vec2::new(self.position.x + 15.0, self.position.y - 30.0)
    }

    fn v3(&self) -> Vec2 {
        Vec2::new(self.position.x + 30.0, self.position.y)
    }

    fn render(&self) {
        if self.health > 0 {
            draw_triangle_lines(self.v1(), self.v2(), self.v3(), 1.0, WHITE)
        }
    }

    fn take_hit(&mut self) {
        if self.iframes == 0 && self.health > 0 {
            self.health -= 1;
            self.iframes = 30;
        }
    }

    fn vertex_positions(&self) -> Vec<Position> {
        let vertex_1: Vec2 = self.v1();
        let vertex_2: Vec2 = self.v2();
        let vertex_3: Vec2 = self.v3();
        vec![
            Position::new(vertex_1.x, vertex_1.y),
            Position::new(vertex_2.x, vertex_2.y),
            Position::new(vertex_3.x, vertex_3.y),
        ]
    }
}

#[derive(Clone)]
struct Laser {
    id: u32,
    position: Position,
    velocity: Velocity,
}
impl Laser {
    fn new(x_pos: f32, y_pos: f32, x_vel: f32, y_vel: f32, id: u32) -> Laser {
        Laser {
            id,
            position: Position::new(x_pos, y_pos),
            velocity: Velocity::new(x_vel, y_vel),
        }
    }

    fn render(&self) {
        draw_line(
            self.position.x,
            self.position.y,
            self.position.x,
            self.position.y - 10.0,
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
    position: Position,
    velocity: Velocity,
    radius: f32,
    rotation: f32,
    health: u32,
    num_sides: u8,
}
impl Asteroid {
    fn new(x_pos: f32, y_pos: f32, x_vel: f32, y_vel: f32, radius: f32, id: u32) -> Asteroid {
        Asteroid {
            id,
            position: Position::new(x_pos, y_pos),
            velocity: Velocity::new(x_vel, y_vel),
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
    center: Position,
    player: Ship,
    asteroids: Vec<Asteroid>,
    asteroid_counter: u32,
    lasers: Vec<Laser>,
    laser_counter: u32,
    score: u32,
}
impl Game {
    fn new() -> Game {
        let width = screen_width();
        let height = screen_height();

        let mut game = Game {
            width,
            height,
            center: Position::new(width / 2.0, height / 2.0),
            asteroids: vec![],
            lasers: vec![],
            player: Ship::new(width / 2.0, height - 30.0),
            score: 0,
            asteroid_counter: 0,
            laser_counter: 0,
        };
        game.generate_asteroids();
        game
    }

    fn reset(&mut self) {
        let width = screen_width();
        let height = screen_height();
        let center: Position = Position::new(width / 2.0, height / 2.0);

        self.asteroids = vec![];
        self.generate_asteroids();
        self.lasers = vec![];
        self.player = Ship::new(center.x, height - 30.0);
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
        if self.player.iframes > 0 {
            self.player.iframes -= 1;
        }

        let mut remove_asteroid_ids: HashSet<u32> = HashSet::new();
        for a in self.asteroids.iter_mut() {
            a.tick(frame_time);

            // destroy offscreen asteroids
            if a.position.x + a.radius > self.width || a.position.y + a.radius > self.height {
                remove_asteroid_ids.insert(a.id);
            }

            // check for collision with player
            for p in self.player.vertex_positions() {
                if p.distance_to(&a.position) < a.radius {
                    self.player.take_hit();
                    remove_asteroid_ids.insert(a.id);
                }
            }
        }

        // check for lasers hitting asteroids
        let mut remove_laser_ids: HashSet<u32> = HashSet::new();
        for l in self.lasers.iter_mut() {
            l.tick(frame_time);

            // check for contact with an asteroid
            for a in self.asteroids.iter_mut() {
                if l.position.distance_to(&a.position) < a.radius {
                    a.take_hit();
                    remove_laser_ids.insert(l.id);
                    if a.health == 0 {
                        remove_asteroid_ids.insert(a.id);
                        self.score += 1;
                    }
                    break;
                }
            }

            // check for offscreen lasers
            if l.position.x > self.width || l.position.y > self.height {
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
    }

    fn generate_asteroids(&mut self) {
        let radius: f32 = 30.0;
        for _ in 0..10 - self.asteroids.len() {
            let x: f32 = gen_range(radius, self.width - radius);
            self.asteroid_counter += 1;
            self.asteroids.push(Asteroid::new(
                x,
                0.0,
                0.0,
                200.0,
                radius,
                self.asteroid_counter,
            ))
        }
    }
}

#[macroquad::main("Asteroids")]
async fn main() {
    // request_new_screen_size(1200.0, 1000.0);

    let mut game = Game::new();

    // cooldown in seconds
    const LASER_COOLDOWN: f32 = 0.3;
    // pixels per second
    const PLAYER_SPEED: f32 = 300.0;

    let mut laser_cooldown_remaining: f32 = 0.0;
    let mut game_started = false;
    let mut game_over = false;

    loop {
        let frame_time: f32 = get_frame_time();

        clear_background(BLACK);
        if !game_started {
            draw_text_h_centered("Asteroids", game.center.y, 50);
            draw_text_h_centered("Press enter to start the game", game.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                game_started = true;
            }
            next_frame().await;
            continue;
        }

        if !game_over {
            let move_distance = PLAYER_SPEED * frame_time;
            // Check for movement input
            if is_key_down(KeyCode::W) {
                game.player.position.y -= move_distance;
            } else if is_key_down(KeyCode::S) {
                game.player.position.y += move_distance;
            } else if is_key_down(KeyCode::A) {
                game.player.position.x -= move_distance;
            } else if is_key_down(KeyCode::D) {
                game.player.position.x += move_distance;
            }

            // Check for firing
            if laser_cooldown_remaining <= 0.0 && is_key_down(KeyCode::Space) {
                game.laser_counter += 1;
                let fired_laser = Laser::new(
                    game.player.position.x + 15.0,
                    game.player.position.y - 30.0,
                    0.0,
                    -400.0,
                    game.laser_counter,
                );
                game.lasers.push(fired_laser);
                laser_cooldown_remaining = LASER_COOLDOWN;
            }

            game.tick(frame_time);
            game.render();

            // Update laser cooldown
            if laser_cooldown_remaining > 0.0 {
                laser_cooldown_remaining -= frame_time;
            }
        }

        if game.player.health == 0 {
            game_over = true;
            draw_text_h_centered("Game Over", game.center.y, 48);
            draw_text_h_centered("Press enter to play again", game.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                game.reset();
                game_over = false;
            }
        } else if game.score == 20 {
            game_over = true;
            draw_text_h_centered("You Win", game.center.y, 48);
            draw_text_h_centered("Press enter to play again", game.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                game.reset();
                game_over = false;
            }
        }

        next_frame().await
    }
}
