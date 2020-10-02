# readchat

a simple program to display twitch chat in your terminal

**note** this doesn't require an oauth token.

---

## usage:

> readchat some_channel

```
readchat 0.3.1

description:
    simply read-only client for a single twitch channel's chat

usage:
    readchat <channel>

flags:
    -h, --help             prints this message
    -v, --version          prints the version
    -d, --debug            use simulated debug stream instead

optional flags:
    -n, --nick-max <int>   the max width before truncation of nicknames
    -b, --buffer-max <int> the number of messages to keep in the redraw queue

arguments:
    <string>               the twitch channel to join
```

---

## keybinds:

| key      | action             |
| -------- | ------------------ |
| `ctrl-c` | quit               |
| `ctrl-r` | force a redraw     |
| `ctrl-d` | delete a line      |
| `<`      | shrink name column |
| `>`      | grow name column   |

---

## demo mode:

pass the `--debug` flag (e.g. `readchat --debug`)

you can set these `env vars` to configure this mode

| var                       | description                                  | default                 |
| ------------------------- | -------------------------------------------- | ----------------------- |
| `READCHAT_UNIQUE`         | how many unique chatters to generate         | **5** (names)           |
| `READCHAT_DURATION_LOWER` | lower bound of random range between messages | **150** (milliseconds)  |
| `READCHAT_DURATION_UPPER` | upper bound of random range between messages | **1500** (milliseconds) |
| `READCHAT_LENGTH_LOWER`   | lower bound of characters per message        | **5** (letters)         |
| `READCHAT_LENGTH_UPPER`   | upper bound of characters per message        | **300** (letters)       |
