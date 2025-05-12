mod sudoku {
    // sudoku-specific machinery really deserves its own module.  I didn't want to make multiple files, though.

    use std::fmt;
    use std::ops::{Index, IndexMut};

    #[derive(Clone, Copy, Debug)]
    pub enum Cell {
        Digit(u8),
        Empty,
    }

    impl Cell {
        pub fn new(digit: u8) -> Result<Self, String> {
            let mut cell: Cell = Cell::Empty;
            cell.set(digit)?;

            Ok(cell)
        }

        pub fn is_empty(&self) -> bool {
            matches!(self, Cell::Empty)
        }

        pub fn set(&mut self, digit: u8) -> Result<(), String> {
            if digit > 9 {
                return Err(format!(
                    "A non-empty Cell can only contain the values from 1 to 9 (inclusive).  You tried to make a Cell from {}.",
                    digit
                ));
            }

            *self = match digit {
                0 => Cell::Empty, // Okay, I do allow this; but using `Cell::empty()` is more explicit.
                _ => Cell::Digit(digit),
            };

            Ok(())
        }

        pub fn clear(&mut self) {
            *self = Cell::Empty;
        }
    }

    impl fmt::Display for Cell {
        // In _normal_ printing of a `Cell`, let's print a blank space instead of `None`.
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Cell::Digit(digit) => write!(f, "{}", digit)?,
                Cell::Empty => write!(f, " ")?,
            };

            Ok(())
        }
    }

    // row, column
    #[derive(Clone, Copy, Debug)]
    pub struct Position(usize);

    impl Position {
        pub fn new(row: usize, column: usize) -> Self {
            Self(row * 9 + column)
        }

        pub fn row(self) -> usize {
            self.0 / 9
        }

        pub fn column(self) -> usize {
            self.0 % 9
        }

        fn value(self) -> usize {
            self.0
        }
    }

    #[derive(Clone)]
    pub struct Board(Vec<Cell>);

    impl Board {
        pub fn new() -> Self {
            Board(vec![Cell::Empty; 81])
        }

        pub fn reset_from(&mut self, other: &Board) {
            // This is essentially a `memcpy`
            self.0.copy_from_slice(other.0.as_slice());
        }
    }

    impl fmt::Display for Board {
        // This is the Rust equivalent of Dave's `print_grid`.
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for row in 0..9 {
                if row % 3 == 0 && row != 0 {
                    writeln!(f, "- - - + - - - + - - -")?;
                }
                for column in 0..9 {
                    if column % 3 == 0 && column != 0 {
                        write!(f, "| ")?;
                    }
                    // `Cell`s know how to print themselves.  `Cell::Empty` prints as a blank space.
                    write!(f, "{} ", self[Position::new(row, column)])?;
                }
                writeln!(f)?;
            }
            writeln!(f)
        }
    }

    impl FromIterator<u8> for Board {
        // If you can produce a list of `u8`s (of the right length), you can make a `Board`.
        // `0`s in that list represent empty cells.  This is a convenience for outside callers.
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
            let data: Vec<Cell> = iter
                .into_iter()
                .map(|digit| Cell::new(digit).unwrap()) // Is `unwrap` the best plan here?
                .collect();

            debug_assert_eq!(
                data.len(),
                81,
                "A Board must have 81 Cells.  You supplied {}.",
                data.len()
            );

            Board(data)
        }
    }

    impl Index<Position> for Board {
        type Output = Cell;

        // If you have `b: Board` and `p: Position`, you can grab the `Cell` at that `Position` with square
        // brackets.  `b[p]`.
        fn index(&self, position: Position) -> &Self::Output {
            &self.0[position.value()]
        }
    }

    impl IndexMut<Position> for Board {
        // Same as `Index`, above, but you start with a mutable `Board` and you get back a mutable `Cell`.
        fn index_mut(&mut self, position: Position) -> &mut Self::Output {
            &mut self.0[position.value()]
        }
    }
}

use parse_int;
use std::time::{Duration, Instant};
use sudoku::{Board, Cell, Position};

fn main() {
    // Testing: did our NonZeroU8 optimization give us what we wanted?
    println!();
    println!("Memory use:");
    println!("  sudoku::Cell is {} byte(s)", size_of::<Cell>());
    println!("  sudoku::Position is {} byte(s)", size_of::<Position>());
    println!(
        "  soduko::Board is {} byte(s), but now it has `Vec`s inside it, so that's not accurate",
        size_of::<Board>()
    );
    println!();

    #[rustfmt::skip]
    let unsolved_board = Board::from_iter([
        0, 0, 5, 0, 2, 0, 6, 0, 0,
        0, 9, 0, 6, 0, 4, 0, 1, 0,
        2, 0, 0, 5, 0, 0, 0, 0, 3,
        0, 0, 6, 0, 3, 0, 0, 0, 0,
        0, 0, 0, 8, 0, 1, 0, 0, 0,
        0, 0, 0, 0, 9, 0, 4, 0, 0,
        3, 0, 0, 0, 0, 2, 0, 0, 7,
        0, 1, 0, 9, 0, 0, 0, 5, 0,
        0, 0, 4, 0, 6, 0, 8, 0, 0,
    ].iter().copied());

    println!("Sudoku solver\n");
    println!("{}", &unsolved_board);

    let number_of_iterations = 1_000_000;
    let iteration_range = 0..number_of_iterations;
    let mut total_backtracks = 0usize;

    let mut board = Board::new();

    let start = Instant::now();
    for _ in iteration_range {
        board.reset_from(&unsolved_board);

        let (solved, backtrack_count) = solve_sudoku(&mut board, 0usize);
        total_backtracks = backtrack_count;
        if !solved {
            println!("Failed to solve sudoku!");
            break;
        }
    }
    let elapsed: Duration = start.elapsed();

    println!("{}", &board);
    println!();
    println!(
        "Rust solved {} iterations in {:?} seconds, backtracking {} times",
        parse_int::format_pretty_dec(number_of_iterations),
        elapsed,
        parse_int::format_pretty_dec(total_backtracks)
    );
}

fn solve_sudoku(board: &mut Board, mut backtrack_count: usize) -> (bool, usize) {
    if let Some(position) = first_empty_cell(board) {
        // If there _was_ an empty `Cell`, let's test all the possible digits in that spot.
        for digit in 1..=9 {
            if is_digit_valid_here(board, digit, position) {
                board[position].set(digit).unwrap();
                let (solved, new_backtrack_count) = solve_sudoku(board, backtrack_count);
                if solved {
                    return (true, new_backtrack_count);
                }
                backtrack_count = new_backtrack_count;
                board[position].clear();
            }
        }

        // We tried every possible digit in that empty `Cell` and none of them were valid;
        // therefore, the puzzle cannot be solved.
        (false, backtrack_count + 1)
    } else {
        // If there _wasn't_ an empty `Cell`, the the puzzle is solved.
        (true, backtrack_count)
    }
}

fn first_empty_cell(board: &Board) -> Option<Position> {
    for row in 0..9 {
        for column in 0..9 {
            let position = Position::new(row, column);
            if board[position].is_empty() {
                return Some(position);
            }
        }
    }

    None
}

fn is_digit_valid_here(board: &Board, digit: u8, position: Position) -> bool {
    // This is where all those iterators I didn't write would come in handy.

    // Check row
    let row_to_check = position.row();
    for column in 0..9 {
        if let Cell::Digit(found_digit) = board[Position::new(row_to_check, column)] {
            if found_digit == digit {
                return false;
            }
        }
    }

    // Check column
    let column_to_check = position.column();
    for row in 0..9 {
        if let Cell::Digit(found_digit) = board[Position::new(row, column_to_check)] {
            if found_digit == digit {
                return false;
            }
        }
    }

    // Check 3x3 box
    let first_row_to_check = position.row() - position.row() % 3;
    let last_row_to_check = first_row_to_check + 3;
    let first_column_to_check = position.column() - position.column() % 3;
    let last_column_to_check = first_column_to_check + 3;

    for row in first_row_to_check..last_row_to_check {
        for column in first_column_to_check..last_column_to_check {
            if let Cell::Digit(found_digit) = board[Position::new(row, column)] {
                if found_digit == digit {
                    return false;
                }
            }
        }
    }

    true
}
