use std::{collections::VecDeque, io::Write};
use std::time::Duration;
use std::io::stdout;
use crossterm::{ExecutableCommand, queue, terminal, cursor, style::{self, Stylize}};
use crossterm::event::{KeyCode, Event};
use std::error::Error;

const BOARD_WIDTH: u16 = 7;
const BOARD_HEIGHT: u16 = 6;


// Holds all state.
struct GameState {
    selection_row: u16,

}

#[derive(Debug, Clone, Copy)]
enum PlayerName {
    Player,
    Computer,
    Player2,
}

trait Game {
    fn is_game_over(&self) -> bool;
    fn who_wins(&self) -> Option<PlayerName>;
    fn take_turn(&mut self, player: PlayerName);
    fn draw_board(&self) -> Result<(), Box<dyn Error>>;
    //fn whose_turn(&self) -> Option<PlayerName>;
    fn new() -> Self;
    fn accept_input(&mut self, event_code: KeyCode);
    fn drop_puck(&self) -> bool; // Tries to drop a marker. Returns true if successful,
                                 // false if column is full
}

impl Game for GameState {
    fn is_game_over(&self) -> bool {
        false
    }
    fn who_wins(&self) -> Option<PlayerName> {
        unimplemented!();
    }
    fn take_turn(&mut self, player: PlayerName) {
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
               style::PrintStyledContent(format!("Connect 4").blue())
               )?;

        // Print puck over slot player is currently selecting.
        queue!(stdout,
               cursor::MoveTo(2 * self.selection_row - 1, 1),
               style::PrintStyledContent("o".green().slow_blink())
               )?;

        // Print game board
        for y in 1..7 {
            for x in 0..6 {
                queue!(stdout,
                       // There are 2 chars per row, hence x * 2.
                       // And we want the board offset by 2 from the title => y + 2
                       cursor::MoveTo(x*2, y + 1),
                       style::Print("| ")
                       )?;

            }
        }
        /*
        for y in 0..40 {
        for x in 0..150 {
          if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
            // in this loop we are more efficient by not flushing the buffer.
            stdout
              .queue(cursor::MoveTo(x,y))?
              .queue(style::PrintStyledContent( "█".blue()))?;
          }
        }
      }
      */
      stdout.flush()?;
      Ok(())
    }

    fn accept_input(&mut self, event_code: KeyCode) {
        match event_code {
           KeyCode::Left => {
               // Move Left
               if self.selection_row > 0 {
                   self.selection_row -= 1;
               }
           },
           KeyCode::Right => {
               if self.selection_row < 7 {
                   self.selection_row += 1;
               }
           },
           KeyCode::Down => {
               self.drop_puck();
           },
           KeyCode::Enter => {
               self.drop_puck();
           },
           _ => {},
        }
    }

    fn drop_puck(&self) -> bool {
        true
    }

    fn new() -> Self
    {
        GameState {
            selection_row: 1
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
        //println!("It is now {:?}'s turn.", current_player);
        /*
        let mut is_turn_over: bool = false;
        while !is_turn_over {
            // `poll()` waits for an `Event` for a given time period
            if crossterm::event::poll(Duration::from_millis(500))? {
                // It's guaranteed that the `read()` won't block when the `poll()`
                // function returns `true`
                match crossterm::event::read()? {
                    Event::Key(event) => {
                        },
                    _ => {},
                }
            } else {
                // Timeout expired and no `Event` is available
            }
            game.draw_board()?;
        }
        std::thread::sleep_ms(1000);
        */
        player_list.rotate_right(1); // Have to do this at the end to avoid borrowing twice

    }
    Ok(())
}
