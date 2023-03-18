# Time Teller

![Written in: 🦀 Rust](https://img.shields.io/badge/Written%20in-🦀%20Rust-success) 
![with: love 🧡](https://img.shields.io/badge/with-love%20🧡-red)

A Discord bot to interact with times, timezones, and those fancy Discord timestamp badges. Features should include:
- understanding a variety of obvious freetext descriptions of relative (date)times
- remembering many Discord users' timezones in a database, and contextualizing any time description they type out according to that
- on request, tell any user what the time described in a selected message would mean relative to _their_ timezone (privately). This would actually be done through the timezone badges, which the Discord client already interprets nicely.
- reply to messages containing time descriptors with such badges, using a few interaction modes (selected by the user who would send the original message):
  - anytime a time descriptor is detected (might be annoying)
  - when the user uses syntax like `` `time 5pm` `` (only these will be pointed out)
  - alternatively, a message with marked time strings like that could be rewritten to use badges instead, similar to what [Runic Babble](https://github.com/ilonachan/runic-babble) does with `` `mdj ` `` script

I don't have the best track record of maintaining my projects even just to completion, and I'm afraid this one might also end up abandoned.

## License

As always, feel free to take whatever you like, but I won't be responsible if anything breaks on your end. (that's MIT right? let's say this code is MIT licensed.)