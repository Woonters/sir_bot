# SirBot

## Overview

This is a bot for adding the "aHaleAndHeartySir" voice from the novelai tts

It is designed for just my private discord server, so I'm relying on the
discord being dead enough that whatever server runs this bot to not get rate
limited.

## TODO / Dev work

https://docs.rs/serenity/0.12.0/serenity/client/struct.Context.html

TODOs:
An internal voice to fall back on since I can run out of generations / more voices 
Editiable join and leave messages 
Turn any println!'s into logging messages 
Move a lot of bot commands to use discords inbuilt command structure 
Stop saving messages to TEMP.mpeg, and have a fall back message / use that internal voice 

