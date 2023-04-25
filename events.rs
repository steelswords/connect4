use std::time::Duration;

use crossterm::event::{poll, read, Event};
fn main() -> crossterm::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        // `poll()` waits for an `Event` for a given time period
        if crossterm::event::poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match crossterm::event::read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => println!("{:?}", event),
                Event::Mouse(event) => println!("{:?}", event),
                #[cfg(feature = "bracketed-paste")]
                Event::Paste(data) => println!("Pasted {:?}", data),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
                _ => {},
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
}
