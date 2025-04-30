# selector-backend
Backend for team selector.

This is a program written in Rust intended to be used with the web frontend. It connects to a database specifcied in the environment and creates teams for a game called Space Marines 5. If you don't know what that is, that's alright.

## Arguments

``` -p --player <player-id> ```
The main player argument. Pass a player ID here to add that player to the main selection pool.

``` -g --game_type <type> ``` Default: sm5-12-player
Specifiy the type of game, this controls how many player and what positions are used.

``` -a --algorithm <algorithm> ``` Default: advanced-selection
Decide which algoritm to use. Generally leave this default for the random yet matching algorithm.

``` -t --team-count <count> ``` Default: 2
How many teams to use, for whatever strange reason.

``` --modifier-position <player-id> <position id> ```
Modify the selector to force a player into a certain position.

``` --modifier-team <player-id> <team-id> ```
Further modify the selector to force a player to be on a certain team.

``` -m --mvp-calculation-mode <mvp-calc-mode> ``` Default: median
Change the way the MVP (and hit diff) are calculated from the player's games. TODO: This name is bad

``` -n -n-games <games> ``` Default: 50
Only use n amount of games for stat (hit diff, MVP) calculation. Currently WIP.
