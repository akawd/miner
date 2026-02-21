use ::rand;
use macroquad::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::{Write, stdout};
use std::time::Instant;
use tiny_http::{Response, Server};

const SIZE: f32 = 20.0;
const CAPTION_HEIGHT: f32 = 30.0;
const WIDTH: u8 = 9;
const HEIGHT: u8 = 9;
const MINES_COUNT: u8 = 10;

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
    map: Vec<Vec<Cell>>,   // game map - 2d vec with cells
    started_at: Instant,   // for timer in UI
    is_time_blocked: bool, // true when game failed or win
    mines_found: u8,       // how may cells are labeled as mines
    to_open_count: u16,    // how many cells must be opened to win the game (mines not included)
    time: usize,           // game time in seconds
    status: String,        // some text status
}

#[derive(Serialize, Deserialize)]
pub struct ApiMessage {
    pub action: String,
    pub x: Option<usize>,
    pub y: Option<usize>,
}

#[derive(Serialize)]
struct GameState {
    width: usize,
    height: usize,
    cells: Vec<Vec<CellState>>,
    time: usize,
    status: String,
    mines_found: u8,
}

#[derive(Serialize)]
struct CellState {
    row: usize,
    col: usize,
    cell: String,
    is_opened: bool,
    is_labeled: bool,
}

impl Game {
    fn new_game() -> Self {
        Game {
            map: gen_map(HEIGHT, WIDTH),
            started_at: Instant::now(),
            is_time_blocked: false,
            mines_found: 0,
            to_open_count: u16::from(WIDTH) * u16::from(HEIGHT) - u16::from(MINES_COUNT),
            time: 0,
            status: String::new(),
        }
    }

    fn to_state(&self) -> GameState {
        let cells: Vec<Vec<CellState>> = self.map
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
                row.iter()
                    .enumerate()
                    .map(|(col_idx, cell)| {
                        let cell_str = if cell.is_opened {
                            match &cell.cell {
                                CellType::Mine => "mine".to_string(),
                                CellType::Empty => "empty".to_string(),
                                CellType::Number(n) => n.to_string(),
                            }
                        } else {
                            "unknown".to_string()
                        };
                        CellState {
                            row: row_idx,
                            col: col_idx,
                            cell: cell_str,
                            is_opened: cell.is_opened,
                            is_labeled: cell.is_labeled,
                        }
                    })
                    .collect()
            })
            .collect();

        GameState {
            width: WIDTH as usize,
            height: HEIGHT as usize,
            cells,
            time: self.time,
            status: self.status.clone(),
            mines_found: self.mines_found,
        }
    }

    fn fail(&mut self) {
        for row in self.map.iter_mut().flat_map(|row| row.iter_mut()) {
            row.is_opened = true;
        }
        self.status = "Fail((".to_string();
        self.is_time_blocked = true;
    }

    fn win(&mut self) {
        for row in self.map.iter_mut().flat_map(|row| row.iter_mut()) {
            row.is_opened = true;
        }
        self.status = "Win!".to_string();
        self.is_time_blocked = true;
    }
}

impl Display for Game {
    /// Provides map and some data.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "Found: {}, to open: {}\n",
                self.mines_found, self.to_open_count
            )
            .as_str(),
        )?;
        for row in &self.map {
            for cell in row {
                match cell {
                    Cell {
                        cell: CellType::Mine,
                        is_labeled: true,
                        ..
                    } => f.write_str("*")?,
                    Cell {
                        cell: CellType::Mine,
                        is_labeled: false,
                        ..
                    } => f.write_str("?")?,
                    Cell {
                        cell: CellType::Empty,
                        ..
                    } => f.write_str(".")?,
                    Cell {
                        cell: CellType::Number(n),
                        ..
                    } => f.write_str(n.to_string().as_str())?,
                }
            }
            f.write_str("\n")?;
        }
        f.write_str("-----------\n")?;
        Ok(())
    }
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
            if matches!(map[row_number][col_number].cell, CellType::Mine) {
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
    let max_rows = game.map.len();
    let max_cols = game.map[0].len();
    let mut to_process: Vec<(usize, usize)> = vec![start_point];

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

                let row_number = row_number as usize;
                let col_number = col_number as usize;
                if game.map[row_number][col_number].is_opened {
                    continue;
                }

                game.map[row_number][col_number].is_opened = true;
                // made so intentionally, "panic" here means some error in logic
                game.to_open_count -= 1;

                if matches!(game.map[row_number][col_number].cell, CellType::Empty)
                    && !(dx == 0 && dy == 0)
                {
                    to_process.push((row_number, col_number));
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
                continue;
            };

            let row_number: isize = coordinates.0 as isize + dx;
            let col_number: isize = coordinates.1 as isize + dy;

            if row_number < 0
                || row_number == max_rows as isize
                || col_number < 0
                || col_number == max_cols as isize
            {
                continue;
            }

            let row_number = row_number as usize;
            let col_number = col_number as usize;

            match game.map[row_number][col_number] {
                Cell {
                    is_labeled: true, ..
                } => {
                    labeled_count += 1;
                }
                _ => (),
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

            if row_number < 0
                || row_number == max_rows as isize
                || col_number < 0
                || col_number == max_cols as isize
            {
                continue;
            }

            let row_number = row_number as usize;
            let col_number = col_number as usize;
            match game.map[row_number][col_number] {
                Cell {
                    is_opened: true, ..
                }
                | Cell {
                    is_labeled: true, ..
                } => (),
                Cell {
                    cell: CellType::Mine,
                    ..
                } => game.fail(),
                Cell {
                    cell: CellType::Empty,
                    ..
                } => open_empties(game, (row_number, col_number)),
                _ => {
                    game.map[row_number][col_number].is_opened = true;
                    // TODO: some logic error here, panics when "game.to_open_count -= 1"
                    game.to_open_count = game.to_open_count.saturating_sub(1);
                }
            }
        }
    }
}

fn handle_input(game: &mut Game) {
    if game.is_time_blocked {
        return;
    }

    let mouse_position = mouse_position();
    let col = (mouse_position.0 / SIZE) as usize;
    let row = ((mouse_position.1 - CAPTION_HEIGHT) / SIZE) as usize;

    // ======= Mouse clicks handlers =======
    if is_mouse_button_pressed(MouseButton::Left) {
        match game.map[row][col] {
            // Ignore click on labeled cell.
            Cell {
                is_labeled: true, ..
            } => (),
            // Click on mine - game failed.
            Cell {
                cell: CellType::Mine,
                ..
            } => {
                game.fail();
            },
            // Empty cell
            Cell {
                cell: CellType::Empty,
                is_opened: false,
                ..
            } => {
                open_empties(game, (row, col));
            },
            Cell {
                cell: CellType::Number(n),
                is_opened: true,
                ..
            } => {
                open_around(game, (row, col), n);
            },
            _ => {
                game.map[row][col].is_opened = true;
                game.to_open_count -= 1;
            }
        };
        if game.to_open_count == 0 {
            game.win()
        };
        println!("{game}");
        stdout().flush().unwrap();
    } else if is_mouse_button_pressed(MouseButton::Right)
        && !game.map[row][col].is_opened {

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
            let x = 0.0 + (col_index as f32) * SIZE;
            let mut y = CAPTION_HEIGHT + (row_index as f32) * SIZE;

            let bg_color = match game.map[row_index][col_index] {
                Cell {
                    cell: CellType::Empty | CellType::Number(_),
                    is_opened: true,
                    ..
                } => WHITE,
                _ => GRAY,
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
                },
                Cell {
                    cell: CellType::Mine,
                    is_opened: true,
                    ..
                } => {
                    draw_text("X", x + (SIZE - mine_center.width) / 2.0, y, 25.0, RED);
                },
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
                },
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
                },
                // Miss anything?
                _ => (),
            }
        }
    }
}

fn draw_status(game: &mut Game) {
    if !game.is_time_blocked {
        game.time = game.started_at.elapsed().as_secs() as usize;
    }

    draw_text(
        format!(
            "{}/{} {:3} {}",
            game.mines_found, MINES_COUNT, game.time, game.status,
        )
        .as_str(),
        20.0,
        20.0,
        20.0,
        RED,
    );

    draw_text("N for new game.", 450.0, 20.0, 20.0, BEIGE);
}

fn start_http_server() {
    std::thread::spawn(move || {
        let server = Server::http("127.0.0.1:8080").unwrap();
        println!("HTTP server started at http://127.0.0.1:8080");
        
        for request in server.incoming_requests() {
            let url = request.url();
            let method = request.method().as_str();
            
            let response = if url == "/json" && method == "GET" {
                let game = GAME.lock().unwrap();
                let state = game.to_state();
                let json = serde_json::to_string(&state).unwrap();
                Response::from_string(json)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                    )
            } else if url.starts_with("/click") && method == "POST" {
                if let Ok(params) = url::Url::parse(&format!("http://localhost{}", url)) {
                    let x: Option<usize> = params.query_pairs().find(|(k, _)| k == "x").map(|(_, v)| v.parse().ok()).flatten();
                    let y: Option<usize> = params.query_pairs().find(|(k, _)| k == "y").map(|(_, v)| v.parse().ok()).flatten();
                    if let (Some(x), Some(y)) = (x, y) {
                        let mut game = GAME.lock().unwrap();
                        handle_api_input(&mut game, &ApiMessage { action: "click".to_string(), x: Some(x), y: Some(y) });
                        let state = game.to_state();
                        let json = serde_json::to_string(&state).unwrap();
                        Response::from_string(json)
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                            )
                    } else {
                        Response::from_string("Missing x or y")
                    }
                } else {
                    Response::from_string("Invalid URL")
                }
            } else if url.starts_with("/flag") && method == "POST" {
                if let Ok(params) = url::Url::parse(&format!("http://localhost{}", url)) {
                    let x: Option<usize> = params.query_pairs().find(|(k, _)| k == "x").map(|(_, v)| v.parse().ok()).flatten();
                    let y: Option<usize> = params.query_pairs().find(|(k, _)| k == "y").map(|(_, v)| v.parse().ok()).flatten();
                    if let (Some(x), Some(y)) = (x, y) {
                        let mut game = GAME.lock().unwrap();
                        handle_api_input(&mut game, &ApiMessage { action: "flag".to_string(), x: Some(x), y: Some(y) });
                        let state = game.to_state();
                        let json = serde_json::to_string(&state).unwrap();
                        Response::from_string(json)
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                            )
                    } else {
                        Response::from_string("Missing x or y")
                    }
                } else {
                    Response::from_string("Invalid URL")
                }
            } else if url == "/restart" && method == "POST" {
                let mut game = GAME.lock().unwrap();
                *game = Game::new_game();
                let state = game.to_state();
                let json = serde_json::to_string(&state).unwrap();
                Response::from_string(json)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                    )
            } else if url == "/" {
                Response::from_string("Minesweeper API:\n  GET  /json       - game state as JSON\n  POST /click?x=0&y=0 - left click (returns state)\n  POST /flag?x=0&y=0  - toggle flag (returns state)\n  POST /restart      - new game (returns state)")
            } else {
                Response::from_string("Not found")
            };
            
            let _ = request.respond(response);
        }
    });
}

use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref GAME: Mutex<Game> = Mutex::new(Game::new_game());
}

#[macroquad::main("Miner")]
async fn main() {
    request_new_screen_size(WIDTH as f32 * SIZE, (HEIGHT as f32) * SIZE + CAPTION_HEIGHT);
    next_frame().await;

    start_http_server();

    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::N) {
            let mut game = GAME.lock().unwrap();
            *game = Game::new_game();
        }

        {
            let mut game = GAME.lock().unwrap();
            handle_input(&mut game);
            draw(&game);
            draw_status(&mut game);
        }

        next_frame().await
    }
}

fn handle_api_input(game: &mut Game, msg: &ApiMessage) {
    match msg.action.as_str() {
        "restart" => {
            *game = Game::new_game();
        },
        "click" => {
            if let (Some(x), Some(y)) = (msg.x, msg.y) {
                if y < HEIGHT as usize && x < WIDTH as usize {
                    let row = y;
                    let col = x;
                    match game.map[row][col] {
                        Cell { is_labeled: true, .. } => (),
                        Cell { cell: CellType::Mine, .. } => game.fail(),
                        Cell { cell: CellType::Empty, is_opened: false, .. } => {
                            open_empties(game, (row, col));
                        },
                        Cell { cell: CellType::Number(n), is_opened: true, .. } => {
                            open_around(game, (row, col), n);
                        },
                        _ => {
                            game.map[row][col].is_opened = true;
                            game.to_open_count = game.to_open_count.saturating_sub(1);
                        }
                    }
                    if game.to_open_count == 0 {
                        game.win();
                    }
                }
            }
        },
        "flag" => {
            if let (Some(x), Some(y)) = (msg.x, msg.y) {
                if y < HEIGHT as usize && x < WIDTH as usize {
                    let row = y;
                    let col = x;
                    if !game.map[row][col].is_opened {
                        game.map[row][col].is_labeled = !game.map[row][col].is_labeled;
                        if game.map[row][col].is_labeled {
                            game.mines_found = game.mines_found.saturating_add(1);
                        } else {
                            game.mines_found = game.mines_found.saturating_sub(1);
                        }
                    }
                }
            }
        },
        _ => {}
    }
}
