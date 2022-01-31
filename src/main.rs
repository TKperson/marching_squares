use crossterm::{
    cursor::{self, MoveTo},
    style::Print,
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

use std::io::{self, stdout, Write, Stdout};
use std::thread::sleep;
use std::time::{Duration, Instant};

const FPS: u64 = 24;

struct Map {
    grid_width: u16,
    grid_height: u16,
    width: i32,
    height: i32,
    fill_char: &'static str,
    n_of_balls: u8,
    ball_radius: (f32, f32),
    ball_speed: (f32, f32),
}

#[derive(Clone, Copy)]
struct Grid {
    top_left: bool,
    top_right: bool,
    bottom_left: bool,
    bottom_right: bool,
}

type TBalls = Vec<Ball>;

trait Balls {
    fn borders_val(&self, x: i32, y: i32) -> f32;

    fn r#move(&mut self, map: &Map);
}

impl Balls for TBalls {
    fn borders_val(&self, x: i32, y: i32) -> f32 {
        let mut total: f32 = 0.0;
        for ball in self {
            total += ball.implicit_f(x, y);
        }
        total
    }

    fn r#move(&mut self, map: &Map) {
        for ball in self {
            ball.r#move(map);
        }
    }
}

// #[derive(Clone, Copy)]
struct Ball {
    radius: f32,
    x_pos: f32,
    y_pos: f32,
    x_velocity: f32,
    y_velocity: f32,
}

impl Ball {
    fn spawn(radius: f32, x_pos: f32, y_pos: f32, x_velocity: f32, y_velocity: f32) -> Ball {
        Ball {
            radius: radius,
            x_pos: x_pos,
            y_pos: y_pos,
            x_velocity: x_velocity,
            y_velocity: y_velocity,
        }
    }

    fn r#move(&mut self, map: &Map) {
        if self.x_pos - self.radius < 0.0 || self.x_pos + self.radius > map.width as f32 {
            self.x_velocity = -self.x_velocity;
        }
        if self.y_pos - self.radius < 0.0 || self.y_pos + self.radius > map.height as f32 {
            self.y_velocity = -self.y_velocity;
        }
        self.x_pos += self.x_velocity;
        self.y_pos += self.y_velocity;
    }

    fn implicit_f(&self, checker_x: i32, checker_y: i32) -> f32 {
        let x = checker_x as f32 - self.x_pos;
        let y = checker_y as f32 - self.y_pos;
        self.radius / (x * x + y * y).sqrt()
    }
    
}

fn get_terminal_size() -> (i32, i32) {
    use terminal_size::{Width, Height, terminal_size};

    if let Some((Width(w), Height(h))) = terminal_size() {
        return (w as i32, h as i32);
    } 
    panic!("Unable to get terminal size");
}

fn init(map: Map) {
    let default_grid = Grid {
        top_left: false,
        top_right: false,
        bottom_left: false,
        bottom_right: false,
    };
    let mut grids = vec![vec![default_grid; map.width as usize]; map.height as usize];
    let mut balls: TBalls = Vec::new();
    let mut frames = 0;
    let mut stdout = stdout();
    let start = Instant::now();
    stdout
        .queue(Clear(ClearType::All))
        .unwrap()
        .queue(cursor::Hide)
        .unwrap();
    

    for i in 0..map.n_of_balls {
        let ball_radius = rand::random::<f32>() * (map.ball_radius.1 - map.ball_radius.0) + map.ball_radius.0;
        balls.push(Ball::spawn(
                ball_radius,
                rand::random::<f32>() * (map.width as f32 - 2.0 * ball_radius) + ball_radius, 
                rand::random::<f32>() * (map.height as f32 - 2.0 * ball_radius) + ball_radius, 
                rand::random::<f32>() * (map.ball_speed.1 - map.ball_speed.0) + map.ball_speed.0, 
                rand::random::<f32>() * (map.ball_speed.1 - map.ball_speed.0) + map.ball_speed.0, 
                ));
    }

    loop { // start animation
        update(&map, &mut grids, &mut balls, &mut stdout, 1.0);
        balls.r#move(&map);
        while (start.elapsed().as_millis() as f64) < (1000 as f64) / (FPS as f64) * (frames as f64) {
            sleep(Duration::from_millis(10));
        }
        frames += 1;
    }

}

fn update(map: &Map, grids: &mut Vec<Vec<Grid>>, balls: &mut TBalls, stdout: &mut Stdout, threshold: f32) {
    let mut buffer = String::new();
    for y in 0..map.height {
        stdout.queue(MoveTo(0, y as u16)).unwrap();

        for x in 0..map.width {
            let mut current = &mut grids[y as usize][x as usize];

            current.top_left     = balls.borders_val(x    , y + 1) < threshold;
            current.top_right    = balls.borders_val(x + 1, y + 1) < threshold;
            current.bottom_left  = balls.borders_val(x    , y    ) < threshold;
            current.bottom_right = balls.borders_val(x + 1, y    ) < threshold;

            if ( current.top_left || // if one side is true then draw
                 current.top_right ||
                 current.bottom_left ||
                 current.bottom_right
               )
                &&
               !(current.top_left && // dont draw if all sides are true
                 current.top_right &&
                 current.bottom_left &&
                 current.bottom_right 
                ) {

                stdout
                    .queue(Print(map.fill_char.to_string()))
                    .unwrap();
            } else {
                stdout
                    .queue(Print(" ".to_string()))
                    .unwrap();
            }
        }
    }

    io::stdout().flush().unwrap();
}


fn main() {
    let (w, h) = get_terminal_size();
    println!("Your terminal is {} cols wide and {} lines tall", w, h);
    init( Map {
        grid_width: 100,
        grid_height: 100,
        width: w,
        height: h,
        fill_char: "A",
        n_of_balls: 3,
        ball_radius: (3.0, 10.0),
        ball_speed: (-2.0, 2.0),
    });
}
