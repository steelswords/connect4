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
    is_turn_over: bool,
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
    fn drop_puck(&mut self); // Tries to drop a marker. Returns true if successful,
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
               style::PrintStyledContent(format!("Connect 4").blue())
               )?;

        // Print puck over slot player is currently selecting.
        queue!(stdout,
               cursor::MoveTo(2 * self.selection_row - 1, 1),
               style::PrintStyledContent("o".green().slow_blink())
               )?;

        // Print game board
        for y in 0..(BOARD_HEIGHT + 1) {
            for x in 0..(BOARD_WIDTH + 1) {
                queue!(stdout,
                       // There are 2 chars per row, hence x * 2.
                       // And we want the board offset by 2 from the title => y + 2
                       cursor::MoveTo(x*2, y + 2),
                       style::Print("| ")
                       )?;

            }
        }
        stdout.flush()?;
        Ok(())
    }

    fn accept_input(&mut self, event_code: KeyCode) {
        match event_code {
           KeyCode::Left => {
               // Move Left
               if self.selection_row > 1 {
                   self.selection_row -= 1;
               }
           },
           KeyCode::Right => {
               if self.selection_row < BOARD_WIDTH {
                   self.selection_row += 1;
               }
           },
           KeyCode::Down => {
               self.drop_puck();
           },
           KeyCode::Enter => {
               self.drop_puck();
           },
           KeyCode::Esc => { exit(0); },
           _ => {},
        }
    }

    fn drop_puck(&mut self) {
        
    }

    fn new() -> Self
    {
        GameState {
            selection_row: 1,
            is_turn_over: false,
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
