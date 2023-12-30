# MIUU WR Checker
> Rust Edition


## Config

```toml
# ./config.toml

database_url = "./db.sqlite"

[discord]
webhooks = [
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