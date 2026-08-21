# a2squery

A tool for extracting A2S query information from Source Dedicated Servers. 

I needed some way to easily get JSON out of my Source Dedicated Server instances. Somehow, there does not exist a single CLI tool for this, and you basically have to use someone's library, download some tool from npm (out of the question on NixOS), or write your own A2S response parser. I did the latter.

# Usage

```text
Usage: a2squery --host <HOST> --port <PORT>

Options:
      --host <HOST>  Server hostname or IP
      --port <PORT>  The server's query port
  -h, --help         Print help
  -V, --version      Print version

Example:
	a2squery --host 127.0.0.1 --port 27015
```

To use `a2squery` simply provide an IP or domain name to the `--host` flag, and a port number to the `--port` flag. The output will look like this:

```json
{
  "protocol_ver": 17,
  "server_name": "Team Fortress",
  "map_name": "cp_snakewater_final1",
  "game_dir": "tf",
  "game_name": "Team Fortress",
  "app_id": 440,
  "current_players": 0,
  "max_players": 24,
  "bots": 0,
  "server_type": "Dedicated",
  "os_type": "Linux",
  "visibility": "Public",
  "vac_enabled": true,
  "game_version": "9925705",
  "extra_data": [
    {
      "GameID": 13085927436482341255
    }
  ]
}
```

Have fun.
