use ::rand;
use macroquad::prelude::*;
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

struct Game {
    map: Vec<Vec<Cell>>,
    started_at: Instant,
    is_time_blocked: bool,
    mines_found: u8,
    to_open_count: u16,
    time: usize,
}

/// Generates the map.
fn gen_map(rows: u8, cols: u8) -> Vec<Vec<Cell>> {
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
    for coordinate in coordinates_map.iter().take(MINES_COUNT as usize) {
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
fn open_empties(game: &mut Game, start_point: (usize, usize)) {
    let mut to_process: Vec<(usize, usize)> = vec![start_point];

    let max_rows = game.map.len();
    let max_cols = game.map[0].len();

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

                if game.map[row_number as usize][col_number as usize].is_opened {
                    continue;
                }

                game.map[row_number as usize][col_number as usize].is_opened = true;
                game.to_open_count -= 1;

                if matches!(
                    game.map[row_number as usize][col_number as usize].cell,
                    CellType::Empty
                ) {
                    to_process.push((row_number as usize, col_number as usize));
                }
            }
        }
    }
}


fn open_around(game: &mut Game, coordinates: (usize, usize), number: u8) {
    let max_rows = game.map.len();
    let max_cols = game.map[0].len();

    let mut labeled_count: u8 = 0;

    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue
            };

            let row_number: isize = coordinates.0 as isize + dx;
            let col_number: isize = coordinates.1 as isize + dy;

            if row_number < 0 || row_number == max_rows as isize || col_number < 0 || col_number == max_cols as isize {
                continue
            }

            let row_number = row_number as usize;
            let col_number = col_number as usize;

            match game.map[row_number][col_number] {
                Cell { is_labeled: true, .. } => {
                    labeled_count += 1;
                },
                _ => ()
            }
        }
    }

    if labeled_count != number {
        return;
    }

    for dx in -1..=1 {
        for dy in -1..=1 {
            let row_number: isize = coordinates.0 as isize + dx;
            let col_number: isize = coordinates.1 as isize + dy;

            if row_number < 0 || row_number == max_rows as isize || col_number < 0 || col_number == max_cols as isize {
                continue
            }

            let row_number = row_number as usize;
            let col_number = col_number as usize;
            match game.map[row_number][col_number] {
                Cell { is_opened: true, .. } | Cell { is_labeled: true, .. } => {},
                Cell { cell: CellType::Mine, .. } => {
                    fail(&mut game.map)
                },
                Cell { cell: CellType::Empty, .. } => {
                    open_empties(game, (row_number, col_number))
                },
                _ => {
                    game.map[row_number][col_number].is_opened = true;
                    game.to_open_count -= 1;
                }
            }

        }
    }
}

fn new_game() -> Game {

    Game {
        map: gen_map(WIDTH, HEIGHT),
        started_at: Instant::now(),
        is_time_blocked: false,
        mines_found: 0,
        to_open_count: u16::from(WIDTH) * u16::from(HEIGHT) - u16::from(MINES_COUNT),
        time: 0,
    }
}


fn handle_input(game: &mut Game) {
    let mouse_position = mouse_position();
    let row = (mouse_position.0 / SIZE) as usize;
    let col = ((mouse_position.1 - CAPTION_HEIGHT) / SIZE) as usize;

    // ======= Mouse clicks handlers =======
    if is_mouse_button_pressed(MouseButton::Left) {
        match game.map[row][col] {
            Cell {
                cell: CellType::Mine,
                ..
            } => {
                // game failed
                fail(&mut game.map);
                //status_text.push_str("Fail(");
                game.is_time_blocked = true;
            }
            Cell {
                cell: CellType::Empty,
                is_opened: false,
                ..
            } => {
                open_empties(game, (row, col));
            }
            Cell {
                cell: CellType::Number(n) , is_opened: true, ..
            } => {
                open_around(game, (row, col), n);
            }
            Cell {
                is_labeled: false, ..
            } => {
                game.map[row][col].is_opened = true;
                game.to_open_count -= 1;
            }
            _ => (),
        }
    } else if is_mouse_button_pressed(MouseButton::Right) {
        game.map[row][col].is_labeled = !game.map[row][col].is_labeled;
        if game.map[row][col].is_labeled {
            game.mines_found = game.mines_found.saturating_add(1);
        } else {
            game.mines_found = game.mines_found.saturating_sub(1);
        }
    }
}

fn draw(game: &Game) {

    let mine_center = measure_text("X", None, 25, 1.0);
    let dot_center = measure_text(" ", None, 25, 1.0);
    let q_center = measure_text("!", None, 25, 1.0);

    for (row_index, row) in game.map.iter().enumerate() {
        for (col_index, _) in row.iter().enumerate() {
            let x = 0.0 + (row_index as f32) * SIZE;
            let mut y = CAPTION_HEIGHT + (col_index as f32) * SIZE;

            let bg_color = match game.map[row_index][col_index] {
                Cell {cell: CellType::Empty | CellType::Number(_), is_opened: true, ..} => GRAY,
                _ => DARKGRAY,
            };
            draw_rectangle(x, y, SIZE, SIZE, bg_color);
            draw_rectangle_lines(x, y, SIZE, SIZE, 1.0, DARKGREEN);


            y += SIZE - 3.0;

            match game.map[row_index][col_index] {
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

    draw_text(
        format!(
            "{}/{} {:3} {}",
            game.mines_found, MINES_COUNT, game.time, ""
        ).as_str(),
        20.0,
        20.0,
        20.0,
        RED,
    );

    draw_text(
        "N for new game.",
        450.0,
        20.0,
        20.0,
        BEIGE,
    );

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
    //let mut to_open_count = u16::from(WIDTH) * u16::from(HEIGHT) - u16::from(MINES_COUNT);
    //let mut status_text = String::new();


    //print_map(map.as_slice());


    request_new_screen_size(WIDTH as f32 * SIZE, (HEIGHT as f32) * SIZE + CAPTION_HEIGHT);
    // from the docs: @"the size in macroquad won’t be updated until the next next_frame().await."
    next_frame().await;

    let mut game = new_game();

    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::N) {
            game = new_game();
        }

        draw(&game);
        handle_input(&mut game);

        if !game.is_time_blocked {
            game.time = game.started_at.elapsed().as_secs() as usize;
        }

        next_frame().await
    }
}
