use macroquad::prelude::*;
use rand::gen_range;

struct Position {
    x: f32,
    y: f32,
}

struct Ship {
    position: Position,
    // rotation: f32,
}
impl Ship {
    fn render(&self) {
        let v1: Vec2 = Vec2::new(self.position.x, self.position.y);
        let v2: Vec2 = Vec2::new(self.position.x + 15.0, self.position.y - 30.0);
        let v3: Vec2 = Vec2::new(self.position.x + 30.0, self.position.y);
        draw_triangle_lines(v1, v2, v3, 1.0, WHITE)
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
    num_sides: u8,
    radius: f32,
    rotation: f32,
}
impl Asteroid {
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
        self.rotation += ROTATION_RATE;
    }
}

const ROTATION_RATE: f32 = 0.5;

struct Space {
    width: f32,
    height: f32,
    asteroids: Vec<Asteroid>,
    player: Ship,
    lasers: Vec<Laser>,
}
impl Space {
    fn render(&self) {
        self.player.render();
        for a in &self.asteroids {
            a.render();
        }
        for l in &self.lasers {
            l.render();
        }
    }

    fn tick(&mut self) {
        for a in self.asteroids.iter_mut() {
            a.tick();

            // Reset position if offscreen
            if a.position.x + a.radius > self.width || a.position.y + a.radius > self.height {
                // a.position.x = 0.0;
                a.position.y = 0.0;
            }
        }
        for l in self.lasers.iter_mut() {
            l.tick();
        }
    }
}

#[macroquad::main("Asteroids")]
async fn main() {
    // request_new_screen_size(1200.0, 1000.0);
    let width = screen_width();
    let height = screen_height();

    // create player
    let player = Ship {
        position: Position {
            x: width / 2.0,
            y: height / 2.0,
        },
    };

    // generate asteroids
    let mut asteroids: Vec<Asteroid> = vec![];
    let radius: f32 = 30.0;
    for _ in 0..10 {
        let x: f32 = gen_range(radius, width - radius);
        let asteroid = Asteroid {
            position: Position { x, y: 0.0 },
            num_sides: 8,
            radius,
            rotation: 0.0,
        };
        asteroids.push(asteroid)
    }

    let lasers: Vec<Laser> = vec![];

    let mut space = Space {
        width,
        height,
        asteroids,
        player,
        lasers,
    };

    let mut laser_cooldown: u32 = 0;
    loop {
        clear_background(BLACK);

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

        if laser_cooldown > 0 {
            laser_cooldown -= 1;
        }

        next_frame().await
    }
}
