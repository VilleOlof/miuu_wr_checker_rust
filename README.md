# MIUU WR Checker
> Rust Edition

MIUU WR Checker *(Marble It Up Ultra! World Record Checker)*  
is a program that regularly checks the MIUU backend  
for new world records, new weekly challenges, posts weekly recaps, downloads all world record replays and saves them all for detailed world record history keeping.  

All being sent to Discord via webhooks embeds.  

MIUU's backend is using the [Parse Platform](https://parseplatform.org/) for communication.  
Which means theres 5 config fields related to Parse, which all must be filled in.  
Without knowing these fields or values this is a pretty useless program to run yourself.  

This exact repo is used in the Official [MIU Discord](http://discord.gg/marbleitup) server and running in production.  
This is mostly public if people wanna see how it works,  
discover bugs or flaws, or grab some of the interesting structs/enums.

## Config
*All fields are required to run the program*  
*Can also be written in JSON or YAML following the same structure*

```toml
# ./config.toml

database_url = "./db.sqlite"
loop_wait_seconds = 120

[discord]
webhooks = [
    "https://discord.com/api/webhooks/.../...",
]
weekly_webhooks = [
    "https://discord.com/api/webhooks/.../...",
]

[parse] # Parse Platform stuff
domain = "www.example.com"
appid = "appid"
class_name = "class"

[parse.weekly]
class_name = "challenge"
class_name_stats = "challenge_stats"
```

### Todos
- Send a DB backup once every 2 weeks ~
- Add proper testing to everything (restricted to offline)
- Properly test a new project how the db handles new times and stuff