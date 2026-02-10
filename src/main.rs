use ::rand;
use macroquad::prelude::*;
use rand::RngExt;
use std::time::Instant;
use rand::seq::SliceRandom;

const SIZE: f32 = 20.0;
const CAPTION_HEIGHT: f32 = 30.0;
const WIDTH: u8 = 30;
const HEIGHT: u8 = 16;
const MINES_COUNT: u8 = 99;

#[derive(Clone)]
pub enum CellType {
    Mine,
    Empty,
    Number(u8),
}

#[derive(Clone)]
pub struct Cell {
    cell: CellType,
    is_opened: bool,
    is_labeled: bool,
}

/// Generates the map.
fn gen_map(rows: u8, cols: u8, mut mines_count: u8) -> Vec<Vec<Cell>> {
    let mut map = vec![
        vec![
            Cell {
                cell: CellType::Empty,
                is_opened: false,
                is_labeled: false,
            };
            cols as usize
        ];
        rows as usize
    ];

    // Place mines.
    let mut rng = rand::rng();

    let mut coordinates_map = map
        .iter()
        .enumerate()
        .flat_map(|(x, row)| row.iter().enumerate().map(move |(y, _)| (x, y)))
        .collect::<Vec<(usize, usize)>>();
    // wow, "use" is not only for import (or reuse), but can completely change the code behaviour
    coordinates_map.shuffle(&mut rng);
    for coordinate in coordinates_map.iter().take(mines_count as usize) {
        map[coordinate.0][coordinate.1].cell = CellType::Mine;
    }

    // Calculate numbers around mines.
    for row_number in 0..map.len() {
        for col_number in 0..map[row_number].len() {
            if matches!(
                map[row_number][col_number],
                Cell {
                    cell: CellType::Mine,
                    ..
                }
            ) {
                continue;
            }

            let mut mines_around_count: u8 = 0;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    // This is current cell, skipping...
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let row_number: isize = row_number as isize + dx;
                    let col_number: isize = col_number as isize + dy;

                    // All which go out of the map bounds are skipped.
                    if row_number < 0
                        || row_number == rows as isize
                        || col_number < 0
                        || col_number == cols as isize
                    {
                        continue;
                    }

                    if let CellType::Mine = map[row_number as usize][col_number as usize].cell {
                        mines_around_count += 1;
                    }
                }
            }

            if mines_around_count > 0 {
                map[row_number][col_number].cell = CellType::Number(mines_around_count);
            }
        }
    }

    map
}

/// Opens empty cells.
///
/// This includes opening recursively more than one cell.
fn open_empties(map: &mut Vec<Vec<Cell>>, start_point: (usize, usize), to_open_count: &mut u16) {
    let mut to_process: Vec<(usize, usize)> = vec![start_point];

    let max_rows = map.len();
    let max_cols = map[0].len();

    while let Some(point) = to_process.pop() {
        for dx in -1..=1 {
            for dy in -1..=1 {
                let row_number: isize = point.0 as isize + dx;
                let col_number: isize = point.1 as isize + dy;

                if row_number < 0
                    || row_number == max_rows as isize
                    || col_number < 0
                    || col_number == max_cols as isize
                {
                    continue;
                }

                if map[row_number as usize][col_number as usize].is_opened {
                    continue;
                }

                map[row_number as usize][col_number as usize].is_opened = true;
                *to_open_count -= 1;

                if matches!(
                    map[row_number as usize][col_number as usize].cell,
                    CellType::Empty
                ) {
                    to_process.push((row_number as usize, col_number as usize));
                }
            }
        }
    }
}

fn fail(map: &mut Vec<Vec<Cell>>) {
    for row in map.iter_mut().flat_map(|row| row.iter_mut()) {
        row.is_opened = true;
    }
}

#[warn(dead_code)]
/// For map debugging.
fn print_map(map: &[Vec<Cell>]) {
    for row in map {
        for cell in row {
            match cell.cell {
                CellType::Mine => print!("*"),
                CellType::Empty => print!("."),
                CellType::Number(n) => print!("{}", n),
            }
        }
        println!();
    }
}

#[macroquad::main("Miner")]
async fn main() {
    let mut mines_found: u8 = 0;
    let mut to_open_count = u16::from(WIDTH) * u16::from(HEIGHT) - u16::from(MINES_COUNT);
    let mut status_text = String::new();

    let mut map = gen_map(WIDTH, HEIGHT, MINES_COUNT);
    //print_map(map.as_slice());

    let mine_center = measure_text("X", None, 25, 1.0);
    let dot_center = measure_text(" ", None, 25, 1.0);
    let q_center = measure_text("!", None, 25, 1.0);

    request_new_screen_size(WIDTH as f32 * SIZE, (HEIGHT as f32) * SIZE + CAPTION_HEIGHT);
    // from the docs: @"the size in macroquad won’t be updated until the next next_frame().await."
    next_frame().await;

    let time = Instant::now();
    let mut block_timer = false;
    let mut game_time = time.elapsed().as_secs_f32().floor();

    loop {
        clear_background(GRAY);

        for (row_index, row) in map.iter().enumerate() {
            for (col_index, _) in row.iter().enumerate() {
                let x = 0.0 + (row_index as f32) * SIZE;
                let mut y = CAPTION_HEIGHT + (col_index as f32) * SIZE;
                draw_rectangle_lines(x, y, SIZE, SIZE, 1.0, DARKGREEN);

                y += SIZE - 3.0;

                match map[row_index][col_index] {
                    Cell {
                        is_labeled: true,
                        is_opened: false,
                        ..
                    } => {
                        draw_text("!", x + (SIZE - q_center.width) / 2.0, y, 25.0, RED);
                    }
                    Cell {
                        cell: CellType::Mine,
                        is_opened: true,
                        ..
                    } => {
                        draw_text("X", x + (SIZE - mine_center.width) / 2.0, y, 25.0, RED);
                    }
                    Cell {
                        cell: CellType::Empty,
                        is_opened: true,
                        ..
                    } => {
                        draw_text(
                            " ",
                            x + (SIZE - dot_center.width) / 2.0,
                            y - 5.0,
                            25.0,
                            BLACK,
                        );
                    }
                    Cell {
                        cell: CellType::Number(n),
                        is_opened: true,
                        ..
                    } => {
                        let color = match n {
                            1 => BLUE,
                            2 => GREEN,
                            3 => RED,
                            4 => PURPLE,
                            5 => MAROON,
                            _ => BLACK,
                        };
                        let n_as_str = n.to_string();
                        let text = n_as_str.as_str();
                        let digit_center = measure_text(text, None, 25, 1.0);
                        draw_text(text, x + (SIZE - digit_center.width) / 2.0, y, 25.0, color);
                    }
                    // Miss anything?
                    _ => (),
                }
            }
        }

        let mouse_position = mouse_position();
        let row = (mouse_position.0 / SIZE) as usize;
        let col = ((mouse_position.1 - CAPTION_HEIGHT) / SIZE) as usize;

        // ======= Mouse clicks handlers =======
        if is_mouse_button_pressed(MouseButton::Left) {
            match map[row][col] {
                Cell {
                    cell: CellType::Mine,
                    ..
                } => {
                    // game failed
                    fail(&mut map);
                    status_text.push_str("Fail(");
                    block_timer = true;
                }
                Cell {
                    cell: CellType::Empty,
                    is_opened: false,
                    ..
                } => {
                    open_empties(&mut map, (row, col), &mut to_open_count);
                }
                Cell {
                    is_labeled: false, ..
                } => {
                    map[row][col].is_opened = true;
                    to_open_count -= 1;
                }
                _ => (),
            }
        } else if is_mouse_button_pressed(MouseButton::Right) {
            map[row][col].is_labeled = !map[row][col].is_labeled;
            if map[row][col].is_labeled {
                mines_found = mines_found.saturating_add(1);
            } else {
                mines_found = mines_found.saturating_sub(1);
            }
        }

        if !block_timer {
            game_time = time.elapsed().as_secs_f32().floor();
        }

        draw_text(
            format!(
                "{}/{} {:3} {}",
                mines_found, MINES_COUNT, game_time, status_text
            )
            .as_str(),
            20.0,
            20.0,
            20.0,
            RED,
        );

        next_frame().await
    }
}
