use macroquad::prelude::*;
use rand::gen_range;

struct Position {
    x: f32,
    y: f32,
}

struct Ship {
    position: Position,
}
impl Ship {
    fn render(&self) {
        let v1: Vec2 = Vec2::new(self.position.x, self.position.y);
        let v2: Vec2 = Vec2::new(self.position.x + 15.0, self.position.y - 30.0);
        let v3: Vec2 = Vec2::new(self.position.x + 30.0, self.position.y);
        draw_triangle_lines(v1, v2, v3, 1.0, WHITE)
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
}
impl Space {
    fn render(&self) {
        self.player.render();
        for a in &self.asteroids {
            a.render();
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
            y: height,
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

    let mut space = Space {
        width,
        height,
        asteroids,
        player,
    };

    loop {
        clear_background(BLACK);

        space.tick();
        space.render();

        next_frame().await
    }
}
