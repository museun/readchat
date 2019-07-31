use crossbeam_channel as channel;
use twitchchat::{commands, Event, Message as TwitchMessage};

mod queue;
mod string;
mod window;
mod writer;

/// Timeout between checks for window resizing
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

/// Size of the backlog
pub const QUEUE_SIZE: usize = 64;

/// Max name length before truncation
pub const NICK_MAX: usize = 11;

/// Separator between the left and right columns
pub const SEPARATOR: &str = " | ";

pub use {
    queue::Queue,
    string::{partition, truncate},
    window::Window,
    writer::BufferedWriter,
};

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let channel = args.next().unwrap_or_else(|| {
        eprintln!("usage: {} <channel>", program);
        std::process::exit(1);
    });

    let (tx, rx) = channel::unbounded();
    let handle = std::thread::spawn(move || {
        let mut window = Window::new(QUEUE_SIZE);
        BufferedWriter::stdout().clear_screen();
        loop {
            channel::select! {
                recv(rx) -> event => {
                    match event {
                        Ok(event) => window.add(event),
                        _ => return,
                    }
                },
                default(TIMEOUT) => { window.check_size() },
            }
        }
    });

    let (name, pass) = twitchchat::ANONYMOUS_LOGIN;
    let client = twitchchat::connect_easy(name, pass).unwrap_or_else(|err| {
        eprintln!("cannot connect: {}", err);
        std::process::exit(1);
    });

    let writer = client.writer();
    for event in client.filter::<commands::PrivMsg>() {
        match event {
            Event::IrcReady(..) => writer.join(&channel).unwrap(),
            Event::Message(TwitchMessage::PrivMsg(msg)) => {
                if tx.send(msg).is_err() {
                    break;
                }
            }
            Event::Error(err) => {
                eprintln!("error from twitch: {}", err);
                break;
            }
            _ => {}
        }
    }

    // TODO wait for ctrl-c here
    drop(tx);

    let _ = handle.join().unwrap();
}
