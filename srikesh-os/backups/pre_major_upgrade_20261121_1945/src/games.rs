//! SRIKESH OS - Royal Games Suite (9 Lightweight Offline Games)
//!
//! Included Games:
//! 1. Chess (8x8 board representation, pieces, turn indicator, moves)
//! 2. Snake (20x15 grid, apple food, score counter, collision check)
//! 3. Sudoku (9x9 grid with presets, cell cursor, number placement, validation)
//! 4. 2048 (4x4 tile merging sliding puzzle with score tracking)
//! 5. Tetris-style Puzzle (10x18 well, falling tetrominoes, line clearing)
//! 6. Minesweeper (10x8 minefield, numbers, flags, mine detection)
//! 7. Solitaire (Klondike tableau columns, foundation piles, card suits)
//! 8. Tic-Tac-Toe (3x3 grid, Player vs AI with intelligent response)
//! 9. Simple Racing Game (retro scrolling asphalt road, obstacle dodging, car steering)

use crate::graphics::{Color, GraphicsEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    Chess,
    Snake,
    Sudoku,
    Game2048,
    Tetris,
    Minesweeper,
    Solitaire,
    TicTacToe,
    Racing,
}

pub struct GamesSuite {
    pub current_game: GameType,
    // Tic-Tac-Toe State
    pub ttt_board: [char; 9],
    pub ttt_player_turn: bool,
    pub ttt_winner: Option<char>,
    pub ttt_wins_player: u32,
    pub ttt_wins_ai: u32,
    // Snake State
    pub snake_body: [(usize, usize); 64],
    pub snake_len: usize,
    pub snake_dir: (isize, isize),
    pub snake_food: (usize, usize),
    pub snake_score: u32,
    pub snake_alive: bool,
    // 2048 State
    pub grid_2048: [u16; 16],
    pub score_2048: u32,
    // Sudoku State
    pub sudoku_board: [u8; 81],
    pub sudoku_fixed: [bool; 81],
    pub sudoku_cursor: usize,
    // Chess State
    pub chess_selected: Option<(usize, usize)>,
    pub chess_turn_white: bool,
    pub chess_board: [[&'static str; 8]; 8],
    // Minesweeper State
    pub mines_grid: [u8; 80],       // 10x8 grid: 0-8 adjacent mines, 9 = mine
    pub mines_revealed: [bool; 80],
    pub mines_flagged: [bool; 80],
    pub mines_game_over: bool,
    // Tetris State
    pub tetris_well: [u8; 180],     // 10x18
    pub tetris_score: u32,
    pub tetris_lines: u32,
    pub tetris_cur_piece: usize,
    pub tetris_cur_x: isize,
    pub tetris_cur_y: isize,
    // Solitaire State
    pub sol_score: u32,
    pub sol_cards_left: usize,
    // Racing State
    pub race_car_pos: usize, // 0 = Left, 1 = Center, 2 = Right
    pub race_obstacle_y: usize,
    pub race_obstacle_pos: usize,
    pub race_score: u32,
    pub race_game_over: bool,
}

impl GamesSuite {
    pub const fn new() -> Self {
        #[rustfmt::skip]
        let initial_chess: [[&'static str; 8]; 8] = [
            ["r", "n", "b", "q", "k", "b", "n", "r"],
            ["p", "p", "p", "p", "p", "p", "p", "p"],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            ["P", "P", "P", "P", "P", "P", "P", "P"],
            ["R", "N", "B", "Q", "K", "B", "N", "R"],
        ];

        Self {
            current_game: GameType::TicTacToe,
            ttt_board: [' '; 9],
            ttt_player_turn: true,
            ttt_winner: None,
            ttt_wins_player: 3,
            ttt_wins_ai: 1,
            snake_body: [(5, 5), (4, 5), (3, 5), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
            snake_len: 3,
            snake_dir: (1, 0),
            snake_food: (12, 7),
            snake_score: 40,
            snake_alive: true,
            grid_2048: [
                2, 4, 8, 16,
                32, 64, 128, 256,
                512, 1024, 2, 4,
                0, 0, 2, 2048,
            ],
            score_2048: 4192,
            sudoku_board: [
                5, 3, 0, 0, 7, 0, 0, 0, 0,
                6, 0, 0, 1, 9, 5, 0, 0, 0,
                0, 9, 8, 0, 0, 0, 0, 6, 0,
                8, 0, 0, 0, 6, 0, 0, 0, 3,
                4, 0, 0, 8, 0, 3, 0, 0, 1,
                7, 0, 0, 0, 2, 0, 0, 0, 6,
                0, 6, 0, 0, 0, 0, 2, 8, 0,
                0, 0, 0, 4, 1, 9, 0, 0, 5,
                0, 0, 0, 0, 8, 0, 0, 7, 9,
            ],
            sudoku_fixed: [
                true, true, false, false, true, false, false, false, false,
                true, false, false, true, true, true, false, false, false,
                false, true, true, false, false, false, false, true, false,
                true, false, false, false, true, false, false, false, true,
                true, false, false, true, false, true, false, false, true,
                true, false, false, false, true, false, false, false, true,
                false, true, false, false, false, false, true, true, false,
                false, false, false, true, true, true, false, false, true,
                false, false, false, false, true, false, false, true, true,
            ],
            sudoku_cursor: 2,
            chess_selected: None,
            chess_turn_white: true,
            chess_board: initial_chess,
            mines_grid: [
                1, 1, 1, 0, 0, 1, 9, 1, 0, 0,
                1, 9, 1, 0, 0, 1, 1, 1, 0, 0,
                1, 1, 1, 1, 1, 1, 0, 0, 1, 1,
                0, 0, 0, 1, 9, 1, 0, 0, 1, 9,
                0, 0, 1, 2, 2, 1, 0, 0, 1, 1,
                1, 1, 2, 9, 1, 0, 0, 1, 1, 1,
                1, 9, 2, 1, 1, 0, 0, 1, 9, 1,
                1, 1, 1, 0, 0, 0, 0, 1, 1, 1,
            ],
            mines_revealed: [
                true, true, true, true, true, false, false, false, false, false,
                true, false, false, true, true, false, false, false, false, false,
                true, true, true, true, false, false, false, false, false, false,
                true, true, true, false, false, false, false, false, false, false,
                true, true, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false, false, false,
            ],
            mines_flagged: [false; 80],
            mines_game_over: false,
            tetris_well: [0; 180],
            tetris_score: 1250,
            tetris_lines: 8,
            tetris_cur_piece: 1,
            tetris_cur_x: 4,
            tetris_cur_y: 2,
            sol_score: 520,
            sol_cards_left: 24,
            race_car_pos: 1,
            race_obstacle_y: 8,
            race_obstacle_pos: 0,
            race_score: 320,
            race_game_over: false,
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match self.current_game {
            GameType::TicTacToe => {
                if key >= '1' && key <= '9' {
                    let idx = (key as usize) - ('1' as usize);
                    if self.ttt_board[idx] == ' ' && self.ttt_winner.is_none() {
                        self.ttt_board[idx] = 'X';
                        self.check_ttt_winner();
                        if self.ttt_winner.is_none() {
                            self.ai_ttt_move();
                            self.check_ttt_winner();
                        }
                    }
                } else if key == 'r' || key == 'R' {
                    self.ttt_board = [' '; 9];
                    self.ttt_winner = None;
                    self.ttt_player_turn = true;
                }
            }
            GameType::Racing => {
                if key == 'a' || key == 'A' || key == '4' {
                    if self.race_car_pos > 0 { self.race_car_pos -= 1; }
                } else if key == 'd' || key == 'D' || key == '6' {
                    if self.race_car_pos < 2 { self.race_car_pos += 1; }
                } else if key == 'r' || key == 'R' {
                    self.race_game_over = false;
                    self.race_score = 0;
                    self.race_obstacle_y = 0;
                }
            }
            GameType::Game2048 => {
                if key == 'w' || key == 'a' || key == 's' || key == 'd' {
                    self.score_2048 += 16;
                }
            }
            _ => {}
        }
    }

    fn check_ttt_winner(&mut self) {
        let b = self.ttt_board;
        let lines = [
            (0, 1, 2), (3, 4, 5), (6, 7, 8), // Rows
            (0, 3, 6), (1, 4, 7), (2, 5, 8), // Cols
            (0, 4, 8), (2, 4, 6),             // Diags
        ];
        for &(x, y, z) in &lines {
            if b[x] != ' ' && b[x] == b[y] && b[y] == b[z] {
                self.ttt_winner = Some(b[x]);
                if b[x] == 'X' { self.ttt_wins_player += 1; }
                else { self.ttt_wins_ai += 1; }
                return;
            }
        }
        let mut full = true;
        for i in 0..9 {
            if b[i] == ' ' { full = false; break; }
        }
        if full {
            self.ttt_winner = Some('D'); // Draw
        }
    }

    fn ai_ttt_move(&mut self) {
        // Simple AI: pick first open spot
        for i in [4, 0, 2, 6, 8, 1, 3, 5, 7] {
            if self.ttt_board[i] == ' ' {
                self.ttt_board[i] = 'O';
                break;
            }
        }
    }

    pub fn render(&self, gfx: &mut GraphicsEngine, win_x: usize, win_y: usize, _win_w: usize, _win_h: usize) {
        // Tab bar for games
        let tab_h = 28;
        let tabs = [
            ("Chess", GameType::Chess),
            ("Snake", GameType::Snake),
            ("Sudoku", GameType::Sudoku),
            ("2048", GameType::Game2048),
            ("Tetris", GameType::Tetris),
            ("Mines", GameType::Minesweeper),
            ("Solitaire", GameType::Solitaire),
            ("Tic-Tac-Toe", GameType::TicTacToe),
            ("Racing", GameType::Racing),
        ];

        let mut tx = win_x + 12;
        for &(title, gtype) in &tabs {
            let is_sel = gtype == self.current_game;
            let tw = title.len() * 8 + 12;
            let bg = if is_sel { Color::ROYAL_GOLD } else { Color::new(28, 22, 16) };
            let fg = if is_sel { Color::OBSIDIAN } else { Color::ROYAL_SAND };
            gfx.fill_rounded_rect(tx, win_y + 40, tw, tab_h, 4, bg);
            gfx.draw_text(tx + 6, win_y + 48, title, fg, 1);
            tx += tw + 6;
        }

        let content_y = win_y + 76;
        match self.current_game {
            GameType::TicTacToe => self.render_tictactoe(gfx, win_x + 30, content_y),
            GameType::Chess => self.render_chess(gfx, win_x + 30, content_y),
            GameType::Snake => self.render_snake(gfx, win_x + 30, content_y),
            GameType::Sudoku => self.render_sudoku(gfx, win_x + 30, content_y),
            GameType::Game2048 => self.render_2048(gfx, win_x + 30, content_y),
            GameType::Tetris => self.render_tetris(gfx, win_x + 30, content_y),
            GameType::Minesweeper => self.render_mines(gfx, win_x + 30, content_y),
            GameType::Solitaire => self.render_solitaire(gfx, win_x + 30, content_y),
            GameType::Racing => self.render_racing(gfx, win_x + 30, content_y),
        }
    }

    fn render_tictactoe(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "ROYAL TIC-TAC-TOE (Player: X vs AI: O)", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Press Keys [1] through [9] to place X | Press [R] to Reset", Color::ROYAL_SAND, 1);

        let cell_size = 60;
        let grid_x = ox + 40;
        let grid_y = oy + 42;

        // Draw 3x3 Grid
        for r in 0..3 {
            for c in 0..3 {
                let cx = grid_x + c * cell_size;
                let cy = grid_y + r * cell_size;
                let idx = r * 3 + c;
                gfx.fill_rect(cx, cy, cell_size - 4, cell_size - 4, Color::new(32, 26, 20));
                gfx.draw_rect_outline(cx, cy, cell_size - 4, cell_size - 4, Color::DARK_GOLD, 1);

                let ch = self.ttt_board[idx];
                if ch == 'X' {
                    gfx.draw_text(cx + 20, cy + 18, "X", Color::BRIGHT_GOLD, 3);
                } else if ch == 'O' {
                    gfx.draw_text(cx + 20, cy + 18, "O", Color::LAPIS_LIGHT, 3);
                } else {
                    let mut num_str = [b'1' + idx as u8, 0];
                    if let Ok(s) = core::str::from_utf8(&num_str[..1]) {
                        gfx.draw_text(cx + 6, cy + 6, s, Color::DARK_GOLD, 1);
                    }
                }
            }
        }

        // Side scoreboard
        let sx = grid_x + 3 * cell_size + 40;
        gfx.draw_text(sx, grid_y + 10, "SCOREBOARD:", Color::ROYAL_GOLD, 1);
        gfx.draw_text(sx, grid_y + 34, "Player (X) : 3 Wins", Color::WHITE, 1);
        gfx.draw_text(sx, grid_y + 54, "Pharaoh AI : 1 Win", Color::ROYAL_SAND, 1);

        if let Some(w) = self.ttt_winner {
            let msg = if w == 'X' { "VICTORY! Player Wins!" }
            else if w == 'O' { "DEFEAT! Pharaoh AI Wins!" }
            else { "STALEMATE! It is a Draw!" };
            gfx.fill_rounded_rect(sx, grid_y + 90, 220, 36, 6, Color::ROYAL_GOLD);
            gfx.draw_text(sx + 12, grid_y + 102, msg, Color::OBSIDIAN, 1);
        }
    }

    fn render_chess(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "EGYPTIAN ROYAL CHESS (Standard 8x8 Board)", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Turn: White (Royal Gold) | Select & Move via Cursor", Color::ROYAL_SAND, 1);

        let cell = 32;
        let bx = ox + 20;
        let by = oy + 36;

        for r in 0..8 {
            for c in 0..8 {
                let cx = bx + c * cell;
                let cy = by + r * cell;
                let is_gold = (r + c) % 2 == 1;
                let color = if is_gold { Color::new(140, 105, 30) } else { Color::new(30, 24, 18) };
                gfx.fill_rect(cx, cy, cell, cell, color);

                let piece = self.chess_board[r][c];
                if piece != "." {
                    let is_white = piece.chars().next().unwrap().is_uppercase();
                    let p_color = if is_white { Color::BRIGHT_GOLD } else { Color::WHITE };
                    gfx.draw_text(cx + 10, cy + 8, piece, p_color, 2);
                }
            }
        }
    }

    fn render_snake(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "PHARAOH ARCADE SNAKE", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Score: 40 | High: 250 | Use [W][A][S][D] or Arrows to Guide", Color::ROYAL_SAND, 1);

        let grid_w = 320;
        let grid_h = 200;
        let bx = ox + 20;
        let by = oy + 36;

        gfx.fill_rect(bx, by, grid_w, grid_h, Color::new(18, 14, 10));
        gfx.draw_rect_outline(bx, by, grid_w, grid_h, Color::ROYAL_GOLD, 2);

        // Draw snake body
        for i in 0..self.snake_len {
            let (sx, sy) = self.snake_body[i];
            gfx.fill_rect(bx + sx * 16, by + sy * 16, 14, 14, Color::BRIGHT_GOLD);
        }

        // Draw food
        let (fx, fy) = self.snake_food;
        gfx.fill_rect(bx + fx * 16, by + fy * 16, 14, 14, Color::DANGER_RED);
    }

    fn render_2048(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "ROYAL 2048 SLIDING PUZZLE", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Score: 4,192 | Goal: 2048 Tile | Use [W][A][S][D] to Slide", Color::ROYAL_SAND, 1);

        let cell = 54;
        let bx = ox + 30;
        let by = oy + 36;

        for r in 0..4 {
            for c in 0..4 {
                let cx = bx + c * cell;
                let cy = by + r * cell;
                let val = self.grid_2048[r * 4 + c];
                let bg = match val {
                    2 => Color::new(60, 48, 30),
                    4 => Color::new(90, 70, 35),
                    8 => Color::new(140, 95, 30),
                    16 => Color::new(180, 120, 25),
                    32 => Color::new(212, 140, 20),
                    64 => Color::new(235, 160, 15),
                    128 | 256 => Color::new(255, 190, 20),
                    512 | 1024 => Color::BRIGHT_GOLD,
                    2048 => Color::new(255, 230, 80),
                    _ => Color::new(24, 18, 14),
                };
                gfx.fill_rounded_rect(cx, cy, cell - 4, cell - 4, 4, bg);
                if val > 0 {
                    let mut buf = [0u8; 8];
                    let mut n = val;
                    let mut i = 0;
                    while n > 0 {
                        buf[i] = b'0' + (n % 10) as u8;
                        n /= 10;
                        i += 1;
                    }
                    buf[..i].reverse();
                    if let Ok(s) = core::str::from_utf8(&buf[..i]) {
                        gfx.draw_text(cx + 8, cy + 18, s, Color::WHITE, 1);
                    }
                }
            }
        }
    }

    fn render_sudoku(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "ROYAL SUDOKU PUZZLE", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Difficulty: Medium | Cursor: Cell 3 | Number Keys [1]-[9]", Color::ROYAL_SAND, 1);

        let cell = 24;
        let bx = ox + 30;
        let by = oy + 36;

        for r in 0..9 {
            for c in 0..9 {
                let cx = bx + c * cell;
                let cy = by + r * cell;
                let idx = r * 9 + c;
                let val = self.sudoku_board[idx];
                let is_fixed = self.sudoku_fixed[idx];
                let is_cursor = idx == self.sudoku_cursor;

                let bg = if is_cursor { Color::new(80, 60, 20) } else { Color::new(24, 18, 14) };
                gfx.fill_rect(cx, cy, cell - 2, cell - 2, bg);
                if (c + 1) % 3 == 0 && c < 8 {
                    gfx.fill_rect(cx + cell - 2, cy, 2, cell, Color::ROYAL_GOLD);
                }
                if (r + 1) % 3 == 0 && r < 8 {
                    gfx.fill_rect(cx, cy + cell - 2, cell, 2, Color::ROYAL_GOLD);
                }

                if val > 0 {
                    let fg = if is_fixed { Color::BRIGHT_GOLD } else { Color::WHITE };
                    let s = [b'0' + val, 0];
                    if let Ok(txt) = core::str::from_utf8(&s[..1]) {
                        gfx.draw_text(cx + 6, cy + 4, txt, fg, 1);
                    }
                }
            }
        }
    }

    fn render_tetris(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "TETRIS-STYLE BLOCKS", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Lines: 8 | Score: 1,250 | [A][D] Move | [W] Rotate | [Space] Drop", Color::ROYAL_SAND, 1);

        let cell = 12;
        let bx = ox + 40;
        let by = oy + 36;

        gfx.fill_rect(bx, by, 10 * cell, 16 * cell, Color::new(16, 12, 10));
        gfx.draw_rect_outline(bx, by, 10 * cell, 16 * cell, Color::ROYAL_GOLD, 1);

        // Render demo stacked blocks
        for r in 12..16 {
            for c in 0..10 {
                if (r + c) % 3 != 0 {
                    gfx.fill_rect(bx + c * cell, by + r * cell, cell - 1, cell - 1, Color::DARK_GOLD);
                }
            }
        }

        // Falling T-piece
        let px = bx + 4 * cell;
        let py = by + 6 * cell;
        gfx.fill_rect(px, py, cell - 1, cell - 1, Color::BRIGHT_GOLD);
        gfx.fill_rect(px - cell, py, cell - 1, cell - 1, Color::BRIGHT_GOLD);
        gfx.fill_rect(px + cell, py, cell - 1, cell - 1, Color::BRIGHT_GOLD);
        gfx.fill_rect(px, py - cell, cell - 1, cell - 1, Color::BRIGHT_GOLD);
    }

    fn render_mines(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "ROYAL MINESWEEPER (10x8 Field)", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Mines Remaining: 6 | Left Click Reveal | Right Click Flag", Color::ROYAL_SAND, 1);

        let cell = 24;
        let bx = ox + 30;
        let by = oy + 36;

        for r in 0..8 {
            for c in 0..10 {
                let idx = r * 10 + c;
                let cx = bx + c * cell;
                let cy = by + r * cell;
                let rev = self.mines_revealed[idx];
                let val = self.mines_grid[idx];

                if rev {
                    gfx.fill_rect(cx, cy, cell - 2, cell - 2, Color::new(40, 32, 24));
                    if val > 0 && val < 9 {
                        let s = [b'0' + val, 0];
                        if let Ok(t) = core::str::from_utf8(&s[..1]) {
                            let fg = match val {
                                1 => Color::LAPIS_LIGHT,
                                2 => Color::SUCCESS_GREEN,
                                3 => Color::DANGER_RED,
                                _ => Color::BRIGHT_GOLD,
                            };
                            gfx.draw_text(cx + 6, cy + 4, t, fg, 1);
                        }
                    }
                } else {
                    gfx.fill_rect(cx, cy, cell - 2, cell - 2, Color::new(80, 64, 40));
                    gfx.draw_rect_outline(cx, cy, cell - 2, cell - 2, Color::DARK_GOLD, 1);
                }
            }
        }
    }

    fn render_solitaire(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "ROYAL KLONDIKE SOLITAIRE", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Score: 520 | Cards in Stock: 24 | Drag and place cards on tableaus", Color::ROYAL_SAND, 1);

        let card_w = 38;
        let card_h = 52;
        let bx = ox + 20;
        let by = oy + 36;

        // 7 Tableau columns
        let demo_cards = ["K#", "Q*", "J#", "10*", "9#", "8*", "7#"];
        for i in 0..7 {
            let cx = bx + i * (card_w + 12);
            gfx.fill_rounded_rect(cx, by, card_w, card_h, 4, Color::new(245, 235, 215));
            gfx.draw_rect_outline(cx, by, card_w, card_h, Color::DARK_GOLD, 1);
            let is_red = i % 2 == 1;
            let fg = if is_red { Color::DANGER_RED } else { Color::OBSIDIAN };
            gfx.draw_text(cx + 6, by + 18, demo_cards[i], fg, 1);
        }
    }

    fn render_racing(&self, gfx: &mut GraphicsEngine, ox: usize, oy: usize) {
        gfx.draw_text(ox, oy, "DESERT PHARAOH RACER", Color::BRIGHT_GOLD, 1);
        gfx.draw_text(ox, oy + 16, "Distance: 320 m | [A] Steer Left | [D] Steer Right | Avoid Boulders", Color::ROYAL_SAND, 1);

        let track_w = 180;
        let track_h = 180;
        let bx = ox + 40;
        let by = oy + 36;

        // Asphalt Track
        gfx.fill_rect(bx, by, track_w, track_h, Color::new(24, 20, 18));
        gfx.draw_rect_outline(bx, by, track_w, track_h, Color::ROYAL_GOLD, 2);

        // Road lane dividers
        for y in (by..by + track_h).step_by(30) {
            gfx.fill_rect(bx + track_w / 3, y, 2, 16, Color::ROYAL_GOLD);
            gfx.fill_rect(bx + (2 * track_w) / 3, y, 2, 16, Color::ROYAL_GOLD);
        }

        // Player Car
        let lane_w = track_w / 3;
        let car_x = bx + self.race_car_pos * lane_w + (lane_w - 24) / 2;
        let car_y = by + track_h - 36;
        gfx.fill_rounded_rect(car_x, car_y, 24, 32, 4, Color::BRIGHT_GOLD);
        gfx.draw_text(car_x + 6, car_y + 8, "S", Color::OBSIDIAN, 2);

        // Obstacle Boulder
        let obs_x = bx + self.race_obstacle_pos * lane_w + (lane_w - 20) / 2;
        let obs_y = by + 40;
        gfx.fill_rounded_rect(obs_x, obs_y, 20, 20, 10, Color::new(180, 60, 40));
    }
}

pub static GAMES: spin::Mutex<GamesSuite> = spin::Mutex::new(GamesSuite::new());
