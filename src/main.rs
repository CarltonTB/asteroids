use macroquad::prelude::*;
use rand::gen_range;

fn draw_text_h_centered(text: &str, y: f32, font_size: u16) {
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    let x = (screen_width() - text_dimensions.width) / 2.0;
    draw_text(text, x, y, font_size as f32, WHITE);
}

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

struct Ship {
    position: Position,
    health: u32,
    iframes: u32,
    // rotation: f32,
}
impl Ship {
    fn new(x: f32, y: f32) -> Ship {
        Ship {
            position: Position::new(x, y),
            health: 3,
            iframes: 30,
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

struct Laser {
    position: Position,
}
impl Laser {
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

    fn tick(&mut self) {
        self.position.y -= 6.0;
    }
}

struct Asteroid {
    position: Position,
    radius: f32,
    rotation: f32,
    health: u32,
    num_sides: u8,
}
impl Asteroid {
    fn new(x: f32, y: f32, radius: f32) -> Asteroid {
        Asteroid {
            position: Position::new(x, y),
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

    fn tick(&mut self) {
        // self.position.x += 1.0;
        self.position.y += 2.0;
        self.rotation += 0.5;
    }

    fn take_hit(&mut self) {
        if self.health > 0 {
            self.health -= 1;
        }
    }
}

struct GameState {
    width: f32,
    height: f32,
    center: Position,
    asteroids: Vec<Asteroid>,
    player: Ship,
    lasers: Vec<Laser>,
    score: u32,
}
impl GameState {
    fn new() -> GameState {
        let width = screen_width();
        let height = screen_height();
        let center: Position = Position::new(width / 2.0, height / 2.0);

        // create player
        let player = Ship::new(center.x, height - 30.0);

        // generate asteroids
        let mut asteroids: Vec<Asteroid> = vec![];
        let radius: f32 = 30.0;
        for _ in 0..10 {
            let x: f32 = gen_range(radius, width - radius);
            asteroids.push(Asteroid::new(x, 0.0, radius))
        }

        let lasers: Vec<Laser> = vec![];

        GameState {
            width,
            height,
            center,
            asteroids,
            lasers,
            player,
            score: 0,
        }
    }

    fn reset(&mut self) {
        let width = screen_width();
        let height = screen_height();
        let center: Position = Position::new(width / 2.0, height / 2.0);

        // generate asteroids
        let mut asteroids: Vec<Asteroid> = vec![];
        let radius: f32 = 30.0;
        for _ in 0..10 {
            let x: f32 = gen_range(radius, width - radius);
            asteroids.push(Asteroid::new(x, 0.0, radius))
        }

        self.asteroids = asteroids;
        self.lasers = vec![];
        self.player = Ship::new(center.x, height - 30.0);
        self.score = 0;
    }

    fn render(&self) {
        draw_text(&format!("Score: {}", self.score), 10.0, 28.0, 28.0, WHITE);

        self.player.render();

        for a in &self.asteroids {
            a.render();
        }
        for l in &self.lasers {
            l.render();
        }
    }

    fn tick(&mut self) {
        if self.player.iframes > 0 {
            self.player.iframes -= 1;
        }

        for a in self.asteroids.iter_mut() {
            a.tick();

            // reset position if offscreen
            if a.position.x + a.radius > self.width || a.position.y + a.radius > self.height {
                // a.position.x = 0.0;
                a.position.y = 0.0;
            }

            // check for collision with player
            for p in self.player.vertex_positions() {
                if p.distance_to(&a.position) < a.radius {
                    self.player.take_hit();
                    break;
                }
            }
        }

        // check for lasers hitting asteroids
        let mut remove_laser_indices: Vec<usize> = vec![];
        for (i, l) in self.lasers.iter_mut().enumerate() {
            l.tick();

            // check for contact with an asteroid
            let mut remove_asteroid_indices: Vec<usize> = vec![];
            for (j, a) in self.asteroids.iter_mut().enumerate() {
                if l.position.distance_to(&a.position) < a.radius {
                    a.take_hit();
                    remove_laser_indices.push(i);
                    if a.health == 0 {
                        remove_asteroid_indices.push(j);
                        self.score += 1;
                    }
                    break;
                }
            }

            for &i in &remove_asteroid_indices {
                self.asteroids.remove(i);
            }

            // check for offscreen lasers
            if l.position.x > self.width || l.position.y > self.height {
                remove_laser_indices.push(i);
            }
        }

        for &i in &remove_laser_indices {
            self.lasers.remove(i);
        }
    }
}

#[macroquad::main("Asteroids")]
async fn main() {
    // request_new_screen_size(1200.0, 1000.0);

    let mut space = GameState::new();
    let mut laser_cooldown: u32 = 0;
    let mut game_started = false;
    let mut game_over = false;

    loop {
        clear_background(BLACK);
        if !game_started {
            draw_text_h_centered("Asteroids", space.center.y, 50);
            draw_text_h_centered("Press enter to start the game", space.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                game_started = true;
            }
            next_frame().await;
            continue;
        }

        if !game_over {
            // Check for movement input
            if is_key_down(KeyCode::W) {
                space.player.position.y -= 2.0;
            } else if is_key_down(KeyCode::S) {
                space.player.position.y += 2.0;
            } else if is_key_down(KeyCode::A) {
                space.player.position.x -= 2.0;
            } else if is_key_down(KeyCode::D) {
                space.player.position.x += 2.0;
            }

            // Check for firing
            if laser_cooldown == 0 && is_key_down(KeyCode::Space) {
                let fired_laser = Laser {
                    position: Position {
                        x: space.player.position.x + 15.0,
                        y: space.player.position.y - 30.0,
                    },
                };
                space.lasers.push(fired_laser);
                laser_cooldown = 30;
            }

            space.tick();
            space.render();

            // Update laser cooldown
            if laser_cooldown > 0 {
                laser_cooldown -= 1;
            }
        }

        if space.player.health == 0 {
            game_over = true;
            draw_text_h_centered("Game Over", space.center.y, 48);
            draw_text_h_centered("Press enter to play again", space.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                space.reset();
                game_over = false;
            }
            next_frame().await;
            continue;
        } else if space.asteroids.len() == 0 {
            game_over = true;
            draw_text_h_centered("You Win", space.center.y, 48);
            draw_text_h_centered("Press enter to play again", space.center.y + 50.0, 28);
            if is_key_down(KeyCode::Enter) {
                space.reset();
                game_over = false;
            }
            next_frame().await;
            continue;
        }

        next_frame().await
    }
}
