use crossbeam_channel as channel;
use twitchchat::{commands, Event, Message as TwitchMessage};

use readchat::*;

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let channel = args.next().unwrap_or_else(|| {
        eprintln!("usage: {} <channel>", program);
        std::process::exit(1);
    });

    let (quit_tx, quit_rx) = channel::bounded(1);
    ctrlc::set_handler(move || {
        let _ = quit_tx.send(());
    })
    .expect("must be able to handle sigquit");

    let (tx, rx) = channel::unbounded();
    let handle = std::thread::spawn(move || {
        assert!(clicolors_control::configure_terminal());

        let mut term = console::Term::stdout();
        let size = {
            let term = term.clone();
            move || term.size()
        };

        Writer::new(&mut term).clear_screen();
        let mut window = Window::new(QUEUE_SIZE, size, &mut term);

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
                return;
            }
            _ => {}
        }
    }

    // wait for ctrl+c
    let _ = quit_rx.recv();

    // signal everything to shut down
    drop(tx);

    // wait for everything to shut down
    let _ = handle.join();
}
