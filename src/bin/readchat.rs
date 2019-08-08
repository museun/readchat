use crossbeam_channel as channel;
use readchat::*;
use twitchchat::{commands, Event, Message as TwitchMessage};

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

    let (writer, handle) = {
        let tx = tx.clone();

        let (name, pass) = twitchchat::ANONYMOUS_LOGIN;
        let client = twitchchat::connect_easy(name, pass).unwrap_or_else(|err| {
            eprintln!("cannot connect: {}", err);
            std::process::exit(1);
        });

        let writer = client.writer();
        let out_writer = writer.clone();

        let handle = std::thread::spawn(move || {
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
        });

        (out_writer, handle)
    };

    enable_ansi();
    let term = std::io::stdout();
    let mut term = term.lock();

    Writer::new(&mut term).clear_screen();
    let mut window = Window::new(QUEUE_SIZE, &mut term);

    loop {
        channel::select! {
            recv(rx) -> event => {
                match event {
                    Ok(event) => window.add(event),
                    _ => return,
                }
            },
            recv(quit_rx) -> _ => {
                writer.shutdown_client();
                drop(tx);
                let _ = handle.join();
                return;
            }
            default(TIMEOUT) => { window.check_size() },
        }
    }
}
