use std::{
    collections::HashSet,
    io::{BufRead as _, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use twitchchat::messages::Privmsg;

use crate::color::Color;

macro_rules! const_array {
    (@count) => { 0 };
    (@count $odd:tt $($a:tt $b:tt)*) => { (const_array!(@count $($a)*) << 1) | 1 };
    (@count $($a:tt $even:tt)*) => { const_array!(@count $($a)*) << 1 };

    ($vis:vis $ident:ident ; $ty:ty [ $($expr:expr),* $(,)? ]) => {
       $vis const $ident: [$ty; const_array!(@count $($expr)*)] = [$($expr),*];
    };
}

const_array! {
    pub DEFAULT_COLORS; Color [
        Color(0x00, 0x00, 0xFF),
        Color(0x8A, 0x2B, 0xE2),
        Color(0x5F, 0x9E, 0xA0),
        Color(0xD2, 0x69, 0x1E),
        Color(0xFF, 0x7F, 0x50),
        Color(0x1E, 0x90, 0xFF),
        Color(0xB2, 0x22, 0x22),
        Color(0xDA, 0xA5, 0x20),
        Color(0x00, 0x80, 0x00),
        Color(0xFF, 0x69, 0xB4),
        Color(0xFF, 0x45, 0x00),
        Color(0xFF, 0x00, 0x00),
        Color(0x2E, 0x8B, 0x57),
        Color(0x00, 0xFF, 0x7F),
        Color(0xAD, 0xFF, 0x2F),
    ]
}

const_array! {
    ADJECTIVES; &str [
        "bad", "bald", "blue", "busy", "cool", "cute", "dark", "dead", "dull", "easy", "evil", "fair",
        "fine", "fit", "good", "hot", "hurt", "ill", "lazy", "odd", "open", "poor", "real", "rich",
        "ripe", "shy", "sore", "sour", "tame", "tart", "vast", "wild", "zany",
    ]
}

const_array! {
    ANIMALS; &str [
        "alpaca", "ant", "ape", "donkey", "baboon", "badger", "bat", "bear", "beaver", "bee", "beetle",
        "bug", "bull", "camel", "cat", "cicada", "clam", "cod", "coyote", "crab", "crow", "deer",
        "dog", "duck", "eel", "elk", "ferret", "fish", "fly", "fox", "frog", "gerbil", "gnat", "gnu",
        "goat", "hare", "hornet", "horse", "hound", "hyena", "impala", "jackal", "koala", "lion",
        "lizard", "llama", "locust", "louse", "mole", "monkey", "moose", "mouse", "mule", "otter",
        "ox", "oyster", "panda", "pig", "pug", "rabbit", "salmon", "seal", "shark", "sheep", "skunk",
        "snail", "snake", "spider", "swan", "tiger", "trout", "turtle", "walrus", "wasp", "weasel",
        "whale", "wolf", "wombat", "worm", "yak", "zebra",
    ]
}

const_array! {
    IPSUM; &str [
        "Lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", "do", "eiusmod", "tempor",
        "incididunt", "ut", "labore", "et", "dolore", "magna", "aliqua", "Ut", "enim", "ad", "minim", "veniam",
        "quis", "nostrud", "exercitation", "ullamco", "laboris", "nisi", "ut", "aliquip", "ex", "ea", "commodo", "consequat",
        "Duis", "aute", "irure", "dolor", "in", "reprehenderit", "in", "voluptate", "velit", "esse", "cillum", "dolore",
        "eu", "fugiat", "nulla", "pariatur", "Excepteur", "sint", "occaecat", "cupidatat", "non", "proident", "sunt", "in",
        "culpa", "qui", "officia", "deserunt", "mollit", "anim", "id", "est", "laboru"
    ]
}

const_array! {
    READY; &str [
        ":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n",
        ":tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n",
        ":tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n",
        ":tmi.twitch.tv 001 justinfan1234 :Welcome, GLHF!\r\n",
        ":tmi.twitch.tv 372 justinfan1234 :You are in a maze of twisty passages, all alike.\r\n",
        ":tmi.twitch.tv 376 justinfan1234 :>\r\n",
    ]
}

const JOIN: &str = ":justinfan1234!justinfan1234@justinfan1234.tmi.twitch.tv JOIN #testing\r\n";

#[derive(Debug, Copy, Clone)]
pub struct DebugOpts {
    pub unique_chatters: usize,
    pub duration_lower: u64,
    pub duration_upper: u64,
    pub length_lower: usize,
    pub length_upper: usize,
}

impl DebugOpts {
    pub fn load() -> Self {
        fn get<T: std::str::FromStr>(key: &str) -> Option<T> {
            std::env::var(key).ok()?.parse().ok()
        }

        Self {
            unique_chatters: get("READCHAT_UNIQUE").unwrap_or(5),
            duration_lower: get("READCHAT_DURATION_LOWER").unwrap_or(150),
            duration_upper: get("READCHAT_DURATION_UPPER").unwrap_or(1500),
            length_lower: get("READCHAT_LENGTH_LOWER").unwrap_or(5),
            length_upper: get("READCHAT_LENGTH_UPPER").unwrap_or(300),
        }
    }
}

// TODO use our color configuraiton
pub fn simulated_chat(opts: DebugOpts) -> impl Iterator<Item = Privmsg<'static>> {
    let addr = make_interesting_chat(opts).unwrap();
    let stream = TcpStream::connect(addr).unwrap();

    twitchchat::Encoder::new(&stream)
        .encode(twitchchat::commands::join("#testing"))
        .unwrap();

    twitchchat::Decoder::new(stream)
        .into_iter()
        .flatten()
        .flat_map(twitchchat::FromIrcMessage::from_irc)
}

// TODO use our color configuraiton
fn make_interesting_chat(opts: DebugOpts) -> anyhow::Result<SocketAddr> {
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

fn wait_for_join(mut io: &TcpStream) -> anyhow::Result<()> {
    for line in &READY {
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

fn garbage_out(io: &mut dyn Write, chatters: &[Chatter], opts: &DebugOpts) -> anyhow::Result<()> {
    let range = opts.duration_lower..opts.duration_upper;

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

fn feed_chat(listener: TcpListener, chatters: Vec<Chatter>, opts: DebugOpts) {
    for socket in listener.incoming().flatten() {
        if let Err(..) = wait_for_join(&socket) {
            continue;
        }

        if let Err(..) = garbage_out(&mut &socket, &chatters, &opts) {
            continue;
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chatter {
    pub(crate) name: String,
    pub(crate) color: Color,
}

impl Chatter {
    // TODO use our color configuration
    pub(crate) fn new() -> Self {
        let mut name = format!(
            "{}{}",
            ADJECTIVES.choose().unwrap(),
            ANIMALS.choose().unwrap(),
        );
        name.extend(
            std::iter::repeat_with(|| fastrand::u8(b'0'..b'9'))
                .take(fastrand::usize(0..5))
                .map(|c| c as char),
        );

        let color = DEFAULT_COLORS.choose().copied().unwrap();
        Self { name, color }
    }

    pub(crate) fn speak(&self, opts: &DebugOpts) -> String {
        let mut len = fastrand::usize(opts.length_lower..opts.length_upper);
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

pub trait RandExt {
    type Output: ?Sized;
    fn choose(&self) -> Option<&Self::Output>;
}

impl<T> RandExt for [T] {
    type Output = T;
    fn choose(&self) -> Option<&Self::Output> {
        self.get(fastrand::usize(0..self.len()))
    }
}
