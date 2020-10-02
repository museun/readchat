use std::{
    collections::HashSet,
    io::{BufRead as _, BufReader, Write as _},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use twitchchat::twitch::color::RGB;

const READY: &[&str] = &[
    ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n",
    ":tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n",
    ":tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n",
    ":tmi.twitch.tv 001 justinfan1234 :Welcome, GLHF!\r\n",
    ":tmi.twitch.tv 372 justinfan1234 :You are in a maze of twisty passages, all alike.\r\n",
    ":tmi.twitch.tv 376 justinfan1234 :>\r\n",
];

const JOIN: &str = ":justinfan1234!justinfan1234@justinfan1234.tmi.twitch.tv JOIN #testing\r\n";

fn wait_for_join(mut io: &TcpStream) -> anyhow::Result<()> {
    for line in READY {
        io.write_all(line.as_bytes())?;
    }

    for line in BufReader::new(io).lines().flatten() {
        if line == "JOIN #testing" {
            io.write_all(JOIN.as_bytes())?;
            break;
        }
    }

    Ok(())
}

fn garbage_out(mut io: &TcpStream, chatters: &[Chatter], opts: &TestingOpts) -> anyhow::Result<()> {
    let range = opts.duration.0 as u64..opts.duration.1 as u64;
    while let Some(chatter) = chatters.choose() {
        write!(
            io,
            "@color={color} :{name}!{name}@{name} PRIVMSG #testing :{msg}\r\n",
            color = chatter.color,
            name = chatter.name,
            msg = chatter.speak(opts)
        )?;

        std::thread::sleep(Duration::from_millis(fastrand::u64(range.clone())));
    }
    Ok(())
}

fn feed_chat(listener: TcpListener, chatters: Vec<Chatter>, opts: TestingOpts) {
    for socket in listener.incoming().flatten() {
        if let Err(..) = wait_for_join(&socket) {
            continue;
        }

        if let Err(..) = garbage_out(&socket, &chatters, &opts) {
            continue;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TestingOpts {
    pub unique_chatters: usize,
    pub duration: (usize, usize),
    pub length: (usize, usize),
}

impl TestingOpts {
    pub fn load() -> Self {
        fn get(key: &str) -> Option<usize> {
            std::env::var(key).ok().and_then(|p| p.parse().ok())
        }

        Self {
            unique_chatters: get("READCHAT_UNIQUE").unwrap_or(5),
            duration: (
                get("READCHAT_DURATION_LOWER").unwrap_or(150),
                get("READCHAT_DURATION_UPPER").unwrap_or(1500),
            ),
            length: (
                get("READCHAT_LENGTH_LOWER").unwrap_or(5),
                get("READCHAT_LENGTH_UPPER").unwrap_or(300),
            ),
        }
    }
}

pub fn make_interesting_chat(opts: TestingOpts) -> anyhow::Result<std::net::SocketAddr> {
    let mut chatters = Vec::with_capacity(opts.unique_chatters);
    let mut seen = HashSet::new();
    for chatter in std::iter::repeat_with(Chatter::new) {
        if seen.insert(chatter.name.clone()) {
            chatters.push(chatter);
        }

        if chatters.len() == opts.unique_chatters {
            break;
        }
    }

    let listener = TcpListener::bind("localhost:0")?;
    let addr = listener.local_addr()?;

    let _ = std::thread::spawn(move || feed_chat(listener, chatters, opts));

    Ok(addr)
}

#[derive(Debug)]
struct Chatter {
    name: String,
    color: RGB,
}

impl Chatter {
    fn new() -> Self {
        let mut name = format!(
            "{}{}",
            ADJECTIVES.choose().unwrap(),
            ANIMALS.choose().unwrap()
        );
        name.push_str(
            &std::iter::repeat_with(|| fastrand::u8(b'0'..b'9'))
                .take(fastrand::usize(0..5))
                .map(|c| c as char)
                .collect::<String>(),
        );

        let (_, color) = twitchchat::twitch::color::twitch_colors()
            .choose()
            .copied()
            .unwrap();

        Self { name, color }
    }

    fn speak(&self, opts: &TestingOpts) -> String {
        let mut len = fastrand::usize(opts.length.0..opts.length.1);
        let mut data = String::new();

        let mut iter = IPSUM.iter().cycle();
        while len > 0 {
            let ipsum = iter.next().unwrap();

            if fastrand::bool() {
                continue;
            }

            data.push_str(ipsum);
            data.push(' ');
            len = len.saturating_sub(ipsum.len() + 1)
        }

        data
    }
}

trait RandExt {
    type Output: ?Sized;
    fn choose(&self) -> Option<&Self::Output>;
}

impl<T> RandExt for [T] {
    type Output = T;
    fn choose(&self) -> Option<&Self::Output> {
        self.get(fastrand::usize(0..self.len()))
    }
}

const ADJECTIVES: &[&str] = &[
    "bad", "bald", "blue", "busy", "cool", "cute", "dark", "dead", "dull", "easy", "evil", "fair",
    "fine", "fit", "good", "hot", "hurt", "ill", "lazy", "odd", "open", "poor", "real", "rich",
    "ripe", "shy", "sore", "sour", "tame", "tart", "vast", "wild", "zany",
];

const ANIMALS: &[&str] = &[
    "alpaca", "ant", "ape", "donkey", "baboon", "badger", "bat", "bear", "beaver", "bee", "beetle",
    "bug", "bull", "camel", "cat", "cicada", "clam", "cod", "coyote", "crab", "crow", "deer",
    "dog", "duck", "eel", "elk", "ferret", "fish", "fly", "fox", "frog", "gerbil", "gnat", "gnu",
    "goat", "hare", "hornet", "horse", "hound", "hyena", "impala", "jackal", "koala", "lion",
    "lizard", "llama", "locust", "louse", "mole", "monkey", "moose", "mouse", "mule", "otter",
    "ox", "oyster", "panda", "pig", "pug", "rabbit", "salmon", "seal", "shark", "sheep", "skunk",
    "snail", "snake", "spider", "swan", "tiger", "trout", "turtle", "walrus", "wasp", "weasel",
    "whale", "wolf", "wombat", "worm", "yak", "zebra",
];

static IPSUM: &[&str] = &[
    "Lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "Ut",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "ut",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "Duis",
    "aute",
    "irure",
    "dolor",
    "in",
    "reprehenderit",
    "in",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "dolore",
    "eu",
    "fugiat",
    "nulla",
    "pariatur",
    "Excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "in",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
];
