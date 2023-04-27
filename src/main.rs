use std::{collections::VecDeque, io::Write};
use std::time::Duration;
use std::io::stdout;
use crossterm::{ExecutableCommand, queue, terminal, cursor, style::{self, Stylize}};
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
}

#[derive(Debug, Clone, Copy)]
enum PlayerName {
    Player,
    Computer,
    Player2,
    None,
}

trait Game {
    fn is_game_over(&self) -> bool;
    fn who_wins(&self) -> Option<PlayerName>;
    fn take_turn(&mut self, player: PlayerName);
    fn draw_board(&self) -> Result<(), Box<dyn Error>>;
    //fn whose_turn(&self) -> Option<PlayerName>;
    fn new() -> Self;
    fn accept_input(&mut self, event_code: KeyCode);
    fn drop_puck(&mut self); // Tries to drop a marker. Returns true if successful,
                                 // false if column is full
    fn debug_print(&mut self, message: String);
}

impl Game for GameState {
    fn is_game_over(&self) -> bool {
        false
    }
    fn who_wins(&self) -> Option<PlayerName> {
        unimplemented!();
    }
    fn take_turn(&mut self, player: PlayerName) {
        // TODO: make this a GameState variable. Modify it in drop_puck()
        let mut is_turn_over: bool = false;
        while !is_turn_over {
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
               style::Print(&self.debug_message)
               )?;

        // Print puck over slot player is currently selecting.
        queue!(stdout,
               cursor::MoveTo(2 * self.selection_col + 1, 1),
               style::PrintStyledContent("o".green().slow_blink())
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
                    Some(PlayerName::Player) => queue!(stdout, style::PrintStyledContent("o".green()))?,
                    _ => {},
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
               println!("Final status of board:\n{:?}", self.board);
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
        for element in column_iter {
            match element.clone() {
                PlayerName::Player => y = y.checked_sub(1).unwrap_or(y),
                PlayerName::Player2 => y = y.checked_sub(1).unwrap_or(y),
                PlayerName::Computer => y = y.checked_sub(1).unwrap_or(y),
                _ => {},
            };
        }

        // TODO: Somehow we've got to indicate if the turn is over or not.
        self.debug_print(format!("Dropping puck at {},{}", self.selection_col, y));
        self.board.set(y as usize, self.selection_col as usize, PlayerName::Player).unwrap();
    }

    fn debug_print(&mut self, message: String) {
        /*
        queue!(stdout(), cursor::SavePosition, cursor::MoveTo(0, 10), style::Print(message), cursor::RestorePosition ).unwrap();
        stdout().flush().unwrap();
        */
        self.debug_message = message.clone();
    }

    fn new() -> Self
    {
        GameState {
            selection_col: 0,
            is_turn_over: false,
            board: Array2D::filled_with(PlayerName::None, BOARD_HEIGHT as usize, BOARD_WIDTH as usize),
            debug_message: String::new(),
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
    // Set up terminal
    // Main loop
    let mut game = GameState::new();
    let mut player_list = VecDeque::from(vec![PlayerName::Player, PlayerName::Computer]);
    crossterm::terminal::enable_raw_mode()?;
    while !game.is_game_over() {
        let current_player = player_list.front().to_owned();
        match current_player {
            Some(player) => game.take_turn(player.clone()),
            _ => exit(1),
        }
        player_list.rotate_right(1); // Have to do this at the end to avoid borrowing twice

    }
    Ok(())
}
