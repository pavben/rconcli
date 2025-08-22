# rconcli

This is an rcon tool intended for big baller admins who operate a large portfolio of Rust servers. Every line you type gets sent to the server as an rcon command, so be careful about typing `quit` as that will exit the server. To disconnect, use Ctrl+D or Ctrl+C.

Grab the latest release from the GitHub repo.

In the current working directory (from which you will run rconcli), configure all your servers in a `servers.yml` file:

```
servers:
  - id: london
    host: london.rustgalaxy.com
    port: 28016
    password: unbreakable3
  - id: paris
    host: paris.rustgalaxy.com
    port: 28016
    password: unbreakable3
  - id: newyork
    host: newyork.rustgalaxy.com
    port: 28016
    password: unbreakable3
```

Server to connect to is specified as the ID from `servers.yml`, and you can even use the shortest unique prefix!
```
rconcli % ./rconcli s
Server lookup error: Ambiguous ID prefix. Multiple matches found: seoul, singapore, sweden, sydney

rconcli % ./rconcli sin
Connecting to singapore.rustgalaxy.com:28016...
Connected
```

Now try typing `status` and pressing enter:
```
status
hostname: RustGalaxy Nexus Singapore
version : 2594 secure (secure mode enabled, connected to Steam3)
map     : Procedural Map
players : 0 (100 max) (0 queued) (0 joining)

id name ping connected addr owner violation kicks entityId
```

