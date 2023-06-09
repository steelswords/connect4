use std::{collections::VecDeque, io::Write};
use std::time::Duration;
use std::thread;
use std::io::stdout;
use crossterm::{ExecutableCommand, queue, terminal, cursor::{self, Hide, Show}, style::{self, Stylize}};
use crossterm::event::{KeyCode, Event};
use std::error::Error;
use array2d::Array2D;

const BOARD_WIDTH: u16 = 7;
const BOARD_HEIGHT: u16 = 6;


// Holds all state.
struct GameState {
    selection_col: u16,
    is_turn_over: bool,
    board: Array2D<PlayerName>, // 0,0 is top left.
    debug_message: String,
    player_list: VecDeque<PlayerName>,
    current_player: PlayerName,
    winner: PlayerName,
}

#[derive(Clone, Copy, PartialEq)]
enum PlayerName {
    Player,
    Computer,
    Player2,
    None,
}

impl std::fmt::Debug for PlayerName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}", match self {
            PlayerName::Player => "Player 1".to_string(),
            PlayerName::Player2 => "Player 2".to_string(),
            PlayerName::Computer => "Computer".to_string(),
            PlayerName::None => "No one".to_string(),
            
        })
    }
}

fn player_to_color(player: &PlayerName) -> style::Color {
    match player {
        PlayerName::None => style::Color::DarkGrey,
        PlayerName::Computer => style::Color::Red,
        PlayerName::Player => style::Color::Green,
        PlayerName::Player2 => style::Color::Blue,
    }
}

trait Game {
    fn is_game_over(&self) -> bool;
    fn who_wins(&self) -> Option<PlayerName>;
    fn take_turn(&mut self);
    fn draw_board(&self) -> Result<(), Box<dyn Error>>;
    //fn whose_turn(&self) -> Option<PlayerName>;
    fn new() -> Self;
    fn accept_input(&mut self, event_code: KeyCode);
    fn drop_puck(&mut self); // Tries to drop a marker. Returns true if successful,
                                 // false if column is full
    fn check_if_game_over(&mut self);
    fn debug_print(&mut self, message: String);
    fn check_helper(&mut self, x: usize, y: usize, same_adjacent_counter: &mut i32, last_player_seen: &mut PlayerName);
}

impl Game for GameState {
    fn is_game_over(&self) -> bool {
        self.winner != PlayerName::None
    }
    fn who_wins(&self) -> Option<PlayerName> {
        match self.winner {
            PlayerName::None => None,
            _ => Some(self.winner)
        }
    }
    fn take_turn(&mut self) {
        // TODO: make this a GameState variable. Modify it in drop_puck()
        self.is_turn_over = false;
        while !self.is_turn_over {
            // `poll()` waits for an `Event` for a given time period
            if crossterm::event::poll(Duration::from_millis(500)).is_ok() {
                // It's guaranteed that the `read()` won't block when the `poll()`
                // function returns `true`
                match crossterm::event::read() {
                    Ok(Event::Key(event)) => {
                            self.accept_input(event.code);
                        },
                    _ => {},
                }
            } else {
                // Timeout expired and no `Event` is available
            }
            self.draw_board();
        }

        self.debug_print(format!("Good turn, {:?}!", self.current_player));
        // Rotate players.
        self.player_list.rotate_right(1); // Have to do this at the end to avoid borrowing twice
        if let Some(current) = self.player_list.front() {
            self.current_player = current.to_owned();
        }
        else {
            self.debug_print(format!("Something bad happened."));
        }

        self.draw_board();
    }

    fn draw_board(&self) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();
        // Clear terminal
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        // Print game title
        queue!(stdout,
               cursor::MoveTo(0,0),
               style::PrintStyledContent(format!("Connect 4").blue()),
               style::Print(" "),
               style::Print(&self.debug_message)
               )?;

        // Print puck over slot player is currently selecting.
        queue!(stdout,
               cursor::MoveTo(2 * self.selection_col + 1, 1),
               style::PrintStyledContent("o".with(player_to_color(&self.current_player)).slow_blink())
               )?;

        // Print game board
        for y in 0..BOARD_HEIGHT {
            for x in 0..(BOARD_WIDTH + 1) { // +1 for the end |
                // Print vertical lines
                queue!(stdout,
                       // There are 2 chars per row, hence x * 2.
                       // And we want the board offset by 2 from the title => y + 2
                       cursor::MoveTo(x*2, y + 2),
                       style::Print("| ")
                       )?;

            }
        }

        // Print all pucks which are there.
        let mut row_index = 0;
        let mut col_index = 0;
        while row_index < self.board.num_rows() {
            while col_index < self.board.num_columns() {
                // Get x of dropped puck.
                let mut x: i32 = col_index as i32 * 2 + 1;
                if x < 0 { x = 0; };

                let y = row_index + 2;

                
                

                queue!(stdout, cursor::MoveTo(x as u16, y as u16))?;
                match self.board.get(row_index, col_index) {
                    None => {},
                    Some(PlayerName::None) => {},
                    Some(player) => {
                            queue!(stdout, style::PrintStyledContent("o".with(player_to_color(&player))))?;
                        }
                };
                col_index += 1;
            }
            row_index += 1;
            col_index = 0;
        }

        stdout.flush()?;
        Ok(())
    }

    fn accept_input(&mut self, event_code: KeyCode) {
        match event_code {
           KeyCode::Left => {
               // Move Left
               if self.selection_col > 0 {
                   self.selection_col -= 1;
               }
           },
           KeyCode::Right => {
               if self.selection_col < BOARD_WIDTH - 1 { // Max is BOARD_WIDTH - 1
                   self.selection_col += 1;
               }
           },
           KeyCode::Down => {
               self.drop_puck();
           },
           KeyCode::Enter => {
               self.drop_puck();
           },
           KeyCode::Esc => {
               //println!("Final status of board:\n{:?}", self.board);
               exit(0);
           },
           _ => {},
        }
    }

    fn drop_puck(&mut self) {
        // Get y of dropped puck.
        let mut y = BOARD_HEIGHT - 1;
        let column_iter = self.board.column_iter(self.selection_col as usize).unwrap();
        // TODO: Refactor this to be neater

        // Starting at the bottom, subtract 1 from y for each puck found in the column.
        for element in column_iter {
            match element.clone() {
                PlayerName::Player | PlayerName::Player2 | PlayerName::Computer => {
                    y = y.checked_sub(1).unwrap_or(y)
                },
                _ => {},
            };
        }

        // If there is not a puck already in x, y, place the puck and end the turn.
        match self.board.get(y as usize, self.selection_col as usize) {
           Some(PlayerName::None) => {
                self.debug_print(format!("Dropping puck at {},{}", self.selection_col, y));
                self.board.set(y as usize, self.selection_col as usize, self.current_player.clone()).unwrap();
                // Check if game is over now.
                self.check_if_game_over();
                self.is_turn_over = true;
            },
           _  => self.debug_print(format!("This column is full. Try again.")),
        }
    }

    // Uses state in last_player_seen and same_adjacent_counter to check if there
    // are four in a row.
    fn check_helper(&mut self, x: usize, y: usize, same_adjacent_counter: &mut i32, last_player_seen: &mut PlayerName) {
        if let Some(element) = self.board.get(y,x) {
            if *element == *last_player_seen && *last_player_seen != PlayerName::None {
                *same_adjacent_counter += 1;
                // If the streak is long enough, we found a winner!
                if *same_adjacent_counter >= 4 {
                    self.winner = element.clone();
                }
            }
            else { // The element is different from last_player_seen, but there is a puck there
                *same_adjacent_counter = 1;
                *last_player_seen = element.clone();
            }
        }
        else { // No puck in the space. Reset everything.
            // We've encountered an empty board grid
            *same_adjacent_counter = 0;
            *last_player_seen = PlayerName::None;
        }
    }

    fn check_if_game_over(&mut self) {
        // NOTE: This could be a lot more efficient if I passed in the last changed
        // location and only checked from there, but that is more complicated and
        // I want to get going with this.

        let mut same_adjacent_counter = 0;
        let mut last_player_seen = PlayerName::None;

        // Check for horizontal streaks.
        // For each row
        for y in 0..self.board.num_rows() {
            // Loop over the row
            for x in 0..self.board.num_columns() {
                self.check_helper(x, y, &mut same_adjacent_counter, &mut last_player_seen);
            }
        }
        if self.winner != PlayerName::None {
            return;
        }
        //
        // Do the same for columns
        //
        same_adjacent_counter = 0;
        last_player_seen = PlayerName::None;
        // For each column
        for x in 0..self.board.num_columns() {
            // Loop over the column
            for y in 0..self.board.num_rows() {
                self.check_helper( x, y, &mut same_adjacent_counter, &mut last_player_seen);
            }
        }
        if self.winner != PlayerName::None {
            return;
        }

        // Check for wins on the diagonals.
        same_adjacent_counter = 0;
        last_player_seen = PlayerName::None;
        // \ Diagonals first
        for x in 0..self.board.num_columns() - 3 {
            // Loop over the column
            for y in 0..self.board.num_rows() - 3 {
                self.check_helper(x, y, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x + 1, y + 1, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x + 2, y + 2, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x + 3, y + 3, &mut same_adjacent_counter, &mut last_player_seen);
                same_adjacent_counter = 0;
                last_player_seen = PlayerName::None;
                if self.winner != PlayerName::None {
                    return;
                }
            }
        }

        // Now for / diagonals.
        same_adjacent_counter = 0;
        last_player_seen = PlayerName::None;
        for x in 3..self.board.num_columns() {
            // Loop over the column
            for y in 0..self.board.num_rows() - 3 {
                self.check_helper(x, y, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x - 1, y + 1, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x - 2, y + 2, &mut same_adjacent_counter, &mut last_player_seen);
                self.check_helper(x - 3, y + 3, &mut same_adjacent_counter, &mut last_player_seen);
                same_adjacent_counter = 0;
                last_player_seen = PlayerName::None;
                if self.winner != PlayerName::None {
                    return;
                }
            }
        }
    }

    fn debug_print(&mut self, message: String) {
        self.debug_message = message.clone();
    }

    fn new() -> Self
    {
        GameState {
            selection_col: 0,
            is_turn_over: false,
            board: Array2D::filled_with(PlayerName::None, BOARD_HEIGHT as usize, BOARD_WIDTH as usize),
            debug_message: String::new(),
            player_list: VecDeque::from(vec![PlayerName::Player, PlayerName::Player2]),
            current_player: PlayerName::Player,
            winner: PlayerName::None,
        }
    }
}

fn exit(code: i32) {
    match crossterm::terminal::disable_raw_mode() {
        Ok(_) => {},
        Err(_) => {
            std::println!("ERROR: Could not exit raw mode.");
            std::process::exit(-1);
        }
    }
    println!("Thanks for playing!");
    std::process::exit(code);
}



fn main() -> Result<(), Box<dyn Error>> {
    let mut game = GameState::new();

    // Set up terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout(), Hide)?;
    game.draw_board().expect("ERROR: Could not clear board!");

    // Main loop
    while !game.is_game_over() {
        // TODO: Abstract away this player tally.
        game.take_turn();

    }
    game.debug_print(format!("Good job! {:?} is the winner!", game.winner));
    game.draw_board()?;
    crossterm::execute!(stdout(), Show)?;
    crossterm::terminal::disable_raw_mode()?;
    stdout().flush()?;
    Ok(())
}
