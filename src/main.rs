use std::{collections::VecDeque, io::Write};
use std::io::stdout;
use crossterm::{ExecutableCommand, queue, terminal, cursor, style::{self, Stylize}};
use std::error::Error;

// Holds the different columns and keeps track of which game pieces are where.
struct GameBoard {

}

// Holds all state.
struct GameState {
    //board: GameBoard
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
    fn take_turn(&self, player: PlayerName);
    fn draw_board(&self) -> Result<(), Box<dyn Error>>;
    //fn whose_turn(&self) -> Option<PlayerName>;
    fn new() -> Self;
}

impl Game for GameState {
    fn is_game_over(&self) -> bool {
        false
    }
    fn who_wins(&self) -> Option<PlayerName> {
        unimplemented!();
    }
    fn take_turn(&self, player: PlayerName) {
        unimplemented!();
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
    fn new() -> Self
    {
        GameState {
            selection_row: 1
        }
    }
}

fn main() -> Result<(), Box<dyn Error>>{
    // Set up terminal
    // Main loop
    let mut game = GameState::new();
    let mut player_list = VecDeque::from(vec![PlayerName::Player, PlayerName::Computer]);
    while !game.is_game_over() {
        let current_player = player_list.front().to_owned();
        //println!("It is now {:?}'s turn.", current_player);
        game.draw_board()?;
        std::thread::sleep_ms(1000);
        player_list.rotate_right(1); // Have to do this at the end to avoid borrowing twice

    }
    Ok(())
}
