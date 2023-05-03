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
    player_list: VecDeque<PlayerName>,
    current_player: PlayerName,
}

#[derive(Debug, Clone, Copy)]
enum PlayerName {
    Player,
    Computer,
    Player2,
    None,
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
    fn debug_print(&mut self, message: String);
}

impl Game for GameState {
    fn is_game_over(&self) -> bool {
        false
    }
    fn who_wins(&self) -> Option<PlayerName> {
        unimplemented!();
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
                self.is_turn_over = true;
            },
           _  => self.debug_print(format!("This column is full. Try again.")),
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

    // Main loop
    while !game.is_game_over() {
        // TODO: Abstract away this player tally.
        game.take_turn();

    }
    Ok(())
}
