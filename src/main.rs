#![allow(dead_code)]

/// usage: rustc -C opt-level=3 main.rs
///    or: cargo run --release

use std::rc::Rc;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize
}

/// shorthand function for creating a Position
fn pos(x: usize, y: usize) -> Position {
    Position {x: x, y: y}
}

impl Position {
    /// gets the index into an indexed sequence that the position represents
    fn raw_index(&self) -> usize {
        self.y * BOARD_WIDTH + self.x
    }
    /// adds a move to the possible moves list if it is legal, or if it should
    fn add_move_if_able(&self, possible_moves: &mut Vec<Position>, pos: Position, state: &GameState) {
        let i = pos.raw_index();
        let potential_region_idx = state.ref_regions[i];
        let ref potential_region: Vec<Position> = state.region_list[potential_region_idx];
        // counts the moves within the potential region
        let mut r = 0;
        for p in potential_region {
            if state.board[p.raw_index()] {
                r += 1;
            }
        }
        if potential_region_idx != state.current_region && !state.board[i] && r < 5 {
            possible_moves.push(pos);
        }
    }

    /// returns a vector containing all legal moves (with stateful optimizations)
    fn all_moves_with_restrictions(&self, state: &GameState) -> Vec<Position> {
        let mut moves = Vec::with_capacity(8);
        let x = self.x;
        let y = self.y;

        if x < BOARD_WIDTH - 1 && y < BOARD_HEIGHT - 2 {
            let p = pos(x+1, y+2);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x < BOARD_WIDTH - 1 && y > 1 {
            let p = pos(x+1, y-2);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x > 0 && y < BOARD_HEIGHT - 2 {
            let p = pos(x-1, y+2);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x > 0 && y > 1 {
            let p = pos(x-1, y-2);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x < BOARD_WIDTH - 2 && y < BOARD_HEIGHT - 1 {
            let p = pos(x+2, y+1);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x < BOARD_WIDTH - 2 && y > 0 {
            let p = pos(x+2, y-1);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x > 1 && y < BOARD_HEIGHT -1 {
            let p = pos(x-2, y+1);
            self.add_move_if_able(&mut moves, p, state);
        }

        if x > 1 && y > 0 {
            let p = pos(x-2, y-1);
            self.add_move_if_able(&mut moves, p, state);
        }

        moves
    }
}

#[derive(Clone)]
struct GameState {
    board: Vec<bool>,
    current_position: Position,
    current_region: usize,
    history: Vec<Position>,
    ref_regions: Rc<[usize; BOARD_WIDTH * BOARD_HEIGHT]>,
    region_list: Rc<Vec<Vec<Position>>>
}

impl GameState {
    fn new(ref_regions: Rc<[usize; BOARD_WIDTH * BOARD_HEIGHT]>, region_list: Rc<Vec<Vec<Position>>>) -> GameState {
        let mut gs = GameState {
            board: vec![false; BOARD_WIDTH*BOARD_HEIGHT],
            current_position: pos(0, 0),
            current_region: 0,
            history: vec![],
            ref_regions: ref_regions,
            region_list: region_list
        };
        gs.add_move(pos(0, 0));
        gs
    }

    /// mutates this gamestate with the new move at position pos.
    fn add_move(&mut self, pos: Position) {
        let i = pos.raw_index();
        self.board[i] = true;
        self.history.push(pos.clone());
        self.current_position = pos;
        self.current_region = self.ref_regions[i];
    }

    /// returns a new game state with the same state of this instance, but with a new move ending at
    /// position pos
    fn into_move(&self, pos: Position) -> GameState {
        let mut new = self.clone();
        new.add_move(pos);
        new
    }

    fn print_board(&self) {
        println!("board - current region: {}", self.current_region);
        for y in 0 .. BOARD_HEIGHT {
            let mut row = vec![];
            for x in 0 .. BOARD_WIDTH {
                row.push(self.board[y*BOARD_WIDTH + x] as u8);
            }
            println!("{:?}", row);
        }
    }

    /// prints the board, with the move number in each position
    fn print_board_with_number(&self) {
        println!("board - current region: {}", self.current_region);
        let mut s: [usize; BOARD_WIDTH * BOARD_HEIGHT] = [0; BOARD_WIDTH * BOARD_HEIGHT];
        for (i, pos) in self.history.iter().enumerate() {
            s[pos.raw_index()] = i + 1;
        }
        for y in 0 .. BOARD_HEIGHT {
            let mut row = Vec::with_capacity(BOARD_WIDTH);
            for x in 0 .. BOARD_WIDTH {
                row.push(s[y*BOARD_WIDTH + x]);
            }
            println!("{:?}", row);
        }
    }

    /// gets all possible next moves for this game state.
    fn all_moves(&self) -> Vec<Position> {
        self.current_position.all_moves_with_restrictions(self)
    }

    /// counts the number of moves within this row
    fn count_in_row(&self, i: usize) -> usize {
        let mut sum = 0;
        for x in 0 .. BOARD_WIDTH {
            if self.board[pos(x, i).raw_index()] {
                sum += 1;
            }
        }
        sum
    }

    /// counts the number of moves within this column
    fn count_in_col(&self, i: usize) -> usize {
        let mut sum = 0;
        for y in 0 .. BOARD_HEIGHT {
            if self.board[pos(i, y).raw_index()] {
                sum += 1;
            }
        }
        sum
    }

    /// Returns true if:
    ///     each column has the same number of moves
    ///     each row has the same number of moves
    ///     each region has the same number of moves
    fn check(&self) -> bool {
        let cl_rows_equal = || {
            let r_count = self.count_in_row(0);
            (1..BOARD_HEIGHT).all(|i| self.count_in_row(i) == r_count)
        };

        let cl_cols_equal = || {
            let c_count = self.count_in_col(0);
            (1..BOARD_WIDTH).all(|i| self.count_in_col(i) == c_count)
        };

        let cl_regions_equal = || {
            let ref first_region = self.region_list[0];
            let mut fr_region_count = 0;
            for pos in first_region {
                if self.board[pos.raw_index()] {
                    fr_region_count += 1;
                }
            }

            self.region_list.iter().all(|region| {
                let mut count = 0;
                for pos in region {
                    if self.board[pos.raw_index()] {
                        count += 1;
                    }
                }
                fr_region_count == count
            })
        };
        cl_rows_equal() && cl_cols_equal() && cl_regions_equal()
    }
}

/// Returns an indexed sequence of predefined regions of a chessboard
fn init_regions() -> Vec<Vec<Position>> {
    let regions: Vec<Vec<Position>> = vec![
        vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(3, 0), pos(3, 1), pos(0, 1), pos(0, 2), pos(0, 3)],
        vec![pos(0, 4), pos(0, 5), pos(0, 6), pos(0, 7), pos(0, 8), pos(0, 9), pos(1, 5)],
        vec![pos(1, 1), pos(1, 2), pos(2, 1), pos(1, 3), pos(1, 4), pos(2, 3), pos(2, 4), pos(3, 4), pos(3, 5), pos(4, 4), pos(5, 4)],
        vec![pos(2, 5), pos(2, 6), pos(1, 6), pos(1, 7), pos(1, 8), pos(2, 8)],
        vec![pos(1, 9), pos(2, 9), pos(3, 9), pos(4, 9), pos(3, 8), pos(3, 7), pos(3, 6), pos(2, 7)],
        vec![pos(2, 2), pos(3, 2), pos(4, 2), pos(5, 2), pos(6, 2), pos(3, 3), pos(4, 3), pos(5, 3), pos(6, 3), pos(7, 3), pos(6, 4)],
        vec![pos(4, 0), pos(5, 0), pos(6, 0), pos(7, 0), pos(8, 0), pos(9, 0), pos(4, 1), pos(5, 1), pos(6, 1), pos(7, 1), pos(8, 1)],
        vec![pos(9, 1), pos(9, 2), pos(9, 3), pos(8, 2), pos(7, 2)],
        vec![pos(8, 3), pos(8, 4), pos(8, 5), pos(9, 4), pos(9, 5)],
        vec![pos(5, 5), pos(6, 5), pos(7, 5), pos(7, 4), pos(7, 6), pos(7, 7), pos(8, 6), pos(9, 6)],
        vec![pos(9, 9), pos(8, 9), pos(7, 9), pos(9, 8), pos(8, 8), pos(7, 8), pos(6, 8), pos(9, 7), pos(8, 7), pos(6, 7), pos(4, 7), pos(6, 6), pos(5, 6), pos(4, 6), pos(4, 5)],
        vec![pos(6, 9), pos(5, 9), pos(5, 8), pos(5, 7), pos(4, 8)]
    ];
    regions
}

/// from the regions sequence, return the board, represented as an array of integers.
/// each value in the array corresponds to the index of the input region sequence it belongs to.
fn get_region_map(regions: &Vec<Vec<Position>>) -> [usize; BOARD_WIDTH*BOARD_HEIGHT] {
    let mut blank = [0; BOARD_WIDTH*BOARD_HEIGHT];
    for (i, v) in regions.iter().enumerate() {
        let region_id = i;
        for p in v {
            blank[p.raw_index()] = region_id.clone();
        }
    }
    blank
}

/// Given an input of Gamestates, return paths to the destination that are possible within {moves_left}
/// moves
fn paths_to(mut states: Vec<GameState>, destination: &Position, moves_left: usize) -> Vec<GameState> {
    if moves_left == 0 {
        states.retain(|ref p| &p.current_position == destination); //mutates in-place
        states
    } else {
        let mut v = Vec::with_capacity(states.len() * 8);
        for f in states {
            for m in f.all_moves() {
                v.push(f.into_move(m.clone()));
            }
        }
        paths_to(v, destination, moves_left - 1)
    }
}

/// these are hints provided in the puzzle definition.
fn init_waypoints() -> Vec<(usize, Position)>{
    let waypoints = vec![
        (1,  pos(0, 0)),
        (4,  pos(1, 2)),
        (7,  pos(5, 3)),
        (10, pos(9, 4)),
        (13, pos(9, 1)),
        (16, pos(5, 4)),
        (19, pos(3, 7)),
        (22, pos(2, 9)),
        (25, pos(2, 8)),
        (28, pos(4, 5)),
        (31, pos(9, 5)),
        (34, pos(8, 5)),
        (37, pos(8, 8)),
        (40, pos(9, 2)),
        (43, pos(6, 0)),
        (46, pos(2, 3)),
        (49, pos(0, 6))
    ];
    waypoints
}

fn main() {
    let regions = init_regions();

    let region_map = Rc::new(get_region_map(&regions));
    let region_list = Rc::new(regions);

    let mut flows = vec![GameState::new(region_map, region_list)];
    let waypoints = init_waypoints();

    for &(_, ref waypoint) in waypoints[1..].iter() {
        let ps = paths_to(flows, waypoint, 3);

        for (_, p) in ps.iter().enumerate() {
            if p.check() {
                println!("bingo!");
                p.print_board();
                return;
            }
        }
        flows = ps;
    }

    // if we get to this point, the solution was not found within waypoints.
    // we're now flying blind.
    let mut add = 1;
    loop {
        println!("unguided: {}     realms: {}", add, flows.len());
        add += 1;
        let mut vec = Vec::with_capacity(flows.len());
        for f in flows {
            let moves = f.all_moves();
            for m in moves {
                vec.push(f.into_move(m));
            }
        }

        for (_, p) in vec.iter().enumerate() {
            if p.check() {
                println!("bingo!");
                p.print_board();
                return;
            }
        }
        flows = vec;
    }
}

// not really important, a debugging sanity check
const REG_SIGILS: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'];
fn print_regions(b: &[usize]) /*where A: Clone + std::fmt::Debug*/ {
    for y in 0 .. BOARD_HEIGHT {
        let mut row = vec![];
        for x in 0 .. BOARD_WIDTH {
            row.push(REG_SIGILS[b[y*BOARD_WIDTH + x]].clone());
        }
        println!("{:?}", row);
    }
}
