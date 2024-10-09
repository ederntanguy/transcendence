# Client-Server communication protocol for the Pong game

Version pre-release 3.

This document formalizes the communication between a Pong client and server over a websocket. It is
a stateful protocol, as there is a set of steps from the connection initial handshake to the
deconnexion. Within each step, a set of messages is defined. They are encoded following the CBOR
specification.


## Coordinate system

The coordinate system origin is the top left corner of the game area.

- The x-axis is horizontal and directed to the right.
- The y-axis is vertical and directed to the bottom.


## Constants

This section defines the sizes, speeds, angles, and other numerical aspects of the protocol.

To accommodate any rendering, the server works with relative distances (incidentally, relative speed
as well). The clients are free to scale them up however they need.

The constant `PI` should be chosen from standard libraries or equivalent, with a 64 bit precision.

#### Game area

The height of the game area being fixed to `1.0`, we introduce the constant `RATIO`, which
contains the aspect ratio of the game area. It is the width divided by the height. Thus, the
width is `1.0 * RATIO = RATIO`. This constant is currently `1.3`.

#### Element sizes

The ball is a square. Its edge is `0.017` long.

The pads are rectangles. Their height is `0.100`. Their width is `0.015`.

#### Element speeds

The ball has a velocity of `1.15` per second.

The pads have a velocity of `1.5` per second.

#### Angles

The amplitude of the service angles around the horizontal axis is `PI / 6` radians.

The amplitude of the pad bounce angle around the horizontal axis is `PI / 3` radians.

#### Numerical protocol values

These values are discussed below, in the section they are relevant in.

The rate at which the server ticks internally and sends updates is `100` times per second.

A client is expected to only send an update when necessary : on user input. A maximum of `20`
times within a second is accepted. A server may queue or ignore messages coming sooner than `50` ms
after the previous received message.


## Chronological steps of the protocol

- A client connects to the server at the websocket URI.
- The client informs the server of its desired game mode.
- The server runs the desired game mode with the client.
- When the protocol for the requested game mode is done, the server closes the connexion.


## Disconnections

Disconnections from remote games are handled by the server as follows :
- During the set-up time before a remote game starts a client has a grace period and can disconnect
  with no consequence. The opposing player is put back in the match-making queue.
- Once a game is playing, a disconnection is a withdrawal. It leads to the opponent winning.

Violations of the protocol are treated like disconnections. Servers close the connection, and handle
the situation as explained above.


## Initial connection

Once a websocket connection has been established between the client and the server, the client
informs the server of its intentions.

This first message contains the protocol's version, the sending user's identification, a request for
a game mode, and optional parameters. A server can close a connection if it doesn't support the
given version. The user identification is whatever is necessary for the back-end to know who is
communicating with them. It is currently the user's username. The game mode is a code defined below.
The parameters field contains extra data needed by the server to satisfy the game mode request.

### Messages

- Hello message  
  Structure : {version: u8, id: text string, game_mode: u8, parameters: byte string}
  - The version field is an unsigned integer, monotonically increasing every version of this spec.
    -  Accepted values : {3}.
  - The id field is the username of the client, encoded as a text string.
  - The game_mode field is the unsigned integer code for the requested game mode.
    - Accepted values : {0, 1}.
    - Meaning :
      - 0 : One-versus-one automatically match-made remote game
      - 1 : Local one-versus-one against a guest
  - The parameters field contains the CBOR-encoded data needed to satisfy the game mode request. Its
    type depends on the requested game mode. The versions are defined below.

##### Parameters

- For remote games (mode 0) and local games (mode 1)  
  Description : there is no parameter for these game modes.  
  Structure : {}


## Game start

Once a client has registered for a game with the server, it waits for the latter to
inform it of a game to be ready. When this happens, the server sends a message about the game. Its
contents depend on the game mode requested. In all cases, it includes a time point at which the game
will start.


### The grace period

Users are allowed to disconnect before the starting time. It does not store a loss in the server. A
message is sent to the client still connected, so that it cancels the game start on its side and
waits for a new game start message.


### Messages

- Remote game start message  
  Description : Used for remote games.  
  Structure : {enemy_username: text string, side: u8, starting_time: u64}
  - The enemy_username field is a plain text string, ready to be displayed.
  - The side field is a code for the client's side.
    - Accepted values : {0, 1}
    - Meaning :
      - 0 : Left
      - 1 : Right
  - The starting_time field is the UTC time point at which the game will start. It is a number
    of milliseconds elapsed since the UNIX epoch, in the UTC time zone.
- Local game start message (game mode 1)  
  Structure : {starting_time: u64}
  - The starting_time field is the UTC time point at which the game will start. It is a number
    of milliseconds elapsed since the UNIX epoch, in the UTC time zone.


- Game start status message  
  Description : Indicates whether the game is starting or the peer has disconnected during the
  grace period.  
  Structure : {status: u8}
  - The status field indicates whether the game will effectively start, or if the client is put
    back in queue.
    - Accepted values {0, 1}
    - Meaning :
      - 0 : The game is starting.
      - 1 : The game is aborted, the client is put back in queue.


## Game

In this phase, the client communicates to the server its input, and the servers communicates the
game state to the client. The client should only send a message when an input update happens, as
explained above. The server will send the game state 100 times per second. Once the game is over,
the connection is closed by the server.

### Messages

##### Common to both local (mode 1) and remote (mode 0 and 2) games

- Server-to-client position update message  
  Description : informs the client of the new positions of the game elements.  
  Structure : {msg_id: u8, left_pad_y: f64, right_pad_y: f64, ball_x: f64, ball_y: f64}
  - The msg_id field is 0.
    - Accepted values : {0}
    - Meaning :
      - 0 : This message is a position update message.
  - The left_pad_y field is the position of the left pad on the vertical axis.
    - Accepted values : [0.0..1.0]
  - The right_pad_y field is the position of the right pad on the vertical axis.
    - Accepted values : [0.0..1.0]
  - The ball_x field is the position of the ball on the horizontal axis.
    - Accepted values : [0.0..RATIO]
  - The ball_y field is the position of the ball on the vertical axis.
    - Accepted values : [0.0..1.0]
- Server-to-client point scored message  
  Description : tells the client a given side won a point, and informs it of the new positions of
  the game elements.  
  Structure : {msg_id: u8, side: u8, left_pad_y: f64, right_pad_y: f64, ball_x: f64, ball_y: f64}
  - The msg_id field is 1.
    - Accepted values : {1}
    - Meaning :
      - 1 : This message is a point scored message.
  - The side field is a code for the side that won the point.
    - Accepted values : {0, 1}
    - Meaning :
      - 0 : Left
      - 1 : Right
  - The left_pad_y field is the post-reset position of the left pad on the vertical axis.
    - Accepted values : [0.0..1.0]
  - The right_pad_y field is the post-reset position of the right pad on the vertical axis.
    - Accepted values : [0.0..1.0]
  - The ball_x field is the post-reset position of the ball on the horizontal axis.
    - Accepted values : [0.0..RATIO]
  - The ball_y field is the post-reset position of the ball on the vertical axis.
    - Accepted values : [0.0..1.0]
- Server-to-client game completed message  
  Description : informs the client the game has been won by a given side reaching maximum points.  
  Structure : {msg_id: u8, side: u8}
  - The msg_id field is 2.
    - Accepted values : {2}
    - Meaning :
      - 2 : This message is a game completed message.
  - The side field is a code for the side that won the point, and thus the game.
    - Accepted values : {0, 1}
    - Meaning :
      - 0 : Left
      - 1 : Right

##### Specific to remote games (mode 0 and 2)

- Client-to-server remote input message  
  Description : contains the current movement of the pad.  
  Structure : {movement: i8}
  - The movement field is a small signed integer representing the direction of the user's pad.
    - Accepted values : {-1, 0, 1}
    - Meaning :
      - -1 : Up
      - 0 : Still
      - +1 : Down
- Server-to-client game aborted message  
  Description : informs the client it has won the game by withdrawal, as the opponent disconnected.  
  Structure : {msg_id: u8}
  - The msg_id field is 3.
    - Accepted values : {3}
    - Meaning :
      - 3 : This message is a game aborted message.

##### Specific to local games (mode 1)

- Client-to-server local input message  
  Description : contains the current movement of one of the pads.  
  Structure : {left_movement: i8, right_movement: i8}
  - The left_movement field is a small signed integer representing the direction of the left player's pad.
    - Accepted values : {-1, 0, 1}
    - Meaning :
      - -1 : Up
      - 0 : Still
      - +1 : Down
  - The right_movement field is a small signed integer representing the direction of the right player's pad.
    - Accepted values : {-1, 0, 1}
    - Meaning :
      - -1 : Up
      - 0 : Still
      - +1 : Down
