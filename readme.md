# SirBot

![The Cheeky sir in question](images/gnome_thumbnail.jpg)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Fritzbox2000/sir_bot/rust.yml)
![X (formerly Twitter) URL](https://img.shields.io/twitter/url?url=https%3A%2F%2Ftwitter.com%2Fwoonters)

## Overview

A TTS bot for discord utilising [NovelAI](https://novelai.net) text to speech voices.

This is generally designed to only run on some personal private discord servers with a couple of friends
Therefore there isn't a huge provision for sharding / dealing with multiple servers using it at once. 
[Serenity](https://github.com/serenity-rs/serenity) does handel sharding etc so upgrading to work across
more servers is possible, and a future goal, if only for practicing coding. 

### A Note on TTS 

NovelAI's TTS is a very open API, it doesn't require setting up an account and is supprisingly easy
to utilise, I would suggest taking care when using it, you get 100 free generations and can get through
them quite quickly. Beyond that it's good manners not to abuse public APIs that are useful and cool :)

### A Note on RTVS 

This bot is *HEAVILY* inspired by the blue gnome from [HLVRAI:Alyx](https://www.youtube.com/watch?v=yaHrneT9BfU)
As far as I can tell along with some other tts apis Trog (one of the members) wrote a bot to do the exact 
same thing. That set of streams is why in particular it defaults to using "aHaleAndHeartySir" as the voice.
Please go check them all out, they are cool and funny and generally a delight. 

## How to Use

Download a copy of the bot (I will make releases in the future)
```sh
> git clone https://github.com/Fritzbox2000/sir_bot.git
```
Set a discord bot ID token in the environment 
```
DISCORD_TOKEN="..."  
```
This token is generated when you create the bot, you can only look at it once before you have to re-generate it 
so make sure to write it down somewhere

Edit `prerecordedtable_example.toml` with user id's and audio clips 
and then rename it to:
`prerecordedtable.toml`

Then build and run 
```sh
cargo run
```

## TODO / Dev work

TODOs:
- An internal voice to fall back on since I can run out of generations / more voices 
- xEditiable join and leave messages 
- Turn any println!'s into logging messages 
- Stop saving messages to TEMP.mpeg, and have a fall back message / use that internal voice 
- Error messages on say_saved
- Play youtube videos
- Pause , Stop etc. controls
- Play music from my library maybe?
