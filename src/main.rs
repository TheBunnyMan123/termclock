use std::{io::{self, Write}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::{Duration, SystemTime}};
use core::f32::consts::{PI, FRAC_PI_2};

use chrono::{DateTime, Local, Timelike};
use crossterm::{cursor, execute, style::{self, Color, Colors}, terminal::{self, ClearType}};

const MINUTE_ANGLE_FACTOR: f32 = (2.0 * PI) / 60.0;
const HOUR_ANGLE_FACTOR: f32 = (2.0 * PI) / 12.0;
const UPPER_HALF_BLOCK: char = '\u{2580}';
const LOWER_HALF_BLOCK: char = '\u{2584}';
const FULL_BLOCK: char = '\u{2588}';
const CIRCLE: [(u8, u8); 40] = [
    (6, 1), (7, 1), (8, 1), (9, 1), (10, 1),
    (4, 2), (5, 2), (11, 2), (12, 2),
    (3, 3), (13, 3),
    (2, 4), (14, 4),
    (2, 5), (14, 5),
    (1, 6), (15, 6),
    (1, 7), (15, 7),
    (1, 8), (15, 8),
    (1, 9), (15, 9),
    (1, 10), (15, 10),
    (2, 11), (14, 11),
    (2, 12), (14, 12),
    (3, 13), (13, 13),
    (4, 14), (5, 14), (11, 14), (12, 14),
    (6, 15), (7, 15), (8, 15), (9, 15), (10, 15)
];

fn get_line(x: u8, y: u8, angl: f32, length: f32, pixels: &mut Vec<(u8, u8)>) {
    let angle = angl - FRAC_PI_2;

    let mut x0 = x as i8;
    let mut y0 = y as i8;

    let x1 = (x as f32 + length * angle.cos()).round() as i8;
    let y1 = (y as f32 + length * angle.sin()).round() as i8;

    let dx = f32::abs((x1 - x0) as f32) as i8;
    let dy = f32::abs((y1 - y0) as f32) as i8;

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = if dx > dy { dx } else { -dy } / 2;
    let mut e2;

    loop {
        println!("{} {}", x0, y0);
        pixels.push((x0 as u8, y0 as u8));

        if x0 == x1 && y0 == y1 {
            break;
        }

        e2 = err;
        if e2 > -dx {
            err -= dy;
            x0 += sx;
        }
        if e2 < dy {
            err += dx;
            y0 += sy;
        }
    }
}

fn main() {
    let mut stdout = io::stdout();
    let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        execute!(io::stdout(), cursor::Show).expect("Failed to show cursor");
        r.store(false, Ordering::SeqCst)
    }).expect("Failed to set SIGKILL handler");
    execute!(stdout, cursor::Hide).expect("Failed to hide cursor");

    execute!(stdout, style::SetColors(Colors::new(Color::DarkGrey, Color::White))).expect("Failed to set colors");

    while running.load(Ordering::SeqCst) {
        let now = SystemTime::now();
        let local: DateTime<Local> = now.into();

        let (_, hours_) = local.hour12();
        let hours = hours_ % 12;
        let minutes = local.minute();

        let mut pixels: Vec<(u8, u8)> = CIRCLE.to_vec();
        get_line(8, 8, hours as f32 * HOUR_ANGLE_FACTOR, 2.8, &mut pixels);
        get_line(8, 8, minutes as f32 * MINUTE_ANGLE_FACTOR, 4.8, &mut pixels);

        let mut char_grid: Vec<Vec<char>> = vec![vec![' '; 8]; 16];

        for (x, y) in pixels {
            let half_y = y / 2;
            let current_char = char_grid[x as usize][half_y as usize];

            let upper_half = y % 2 == 0;

            char_grid[x as usize][half_y as usize] = match (current_char, upper_half) {
                (' ', true) => UPPER_HALF_BLOCK,
                (' ', false) => LOWER_HALF_BLOCK,
                (UPPER_HALF_BLOCK, false) => FULL_BLOCK,
                (LOWER_HALF_BLOCK, true) => FULL_BLOCK,
                (char, _) => char
            };
        }

        execute!(stdout, terminal::Clear(ClearType::All)).expect("Failed to clear screen");
        for (x, y_list) in char_grid.iter().enumerate() {
            for (y, char) in y_list.iter().enumerate() {
                execute!(stdout, cursor::MoveTo(x as u16, y as u16)).expect("Failed to move cursor");
                write!(stdout, "{}", char).expect("Failed to write to STDOUT");
            }
        }
        stdout.flush().expect("Failed to flush STDOUT");

        thread::sleep(Duration::from_millis(500));
    }
}

