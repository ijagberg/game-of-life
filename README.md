# game-of-life

Rust implementation of Conway's Game of Life, using [ggez](https://github.com/ggez/ggez)

# Controls

* Click anywhere to create a living cell at that location, or to kill a living cell. 
* Click while holding shift to move the camera around.
* Scroll out/in to change zoom level.
* Space to pause simulation.

# Parameters

* -i <FILE>, sets the initial state of the game to the plaintext formatted state in <FILE> (default="resources/default.txt")
* -r <INT>, sets the number of updates per second to <INT> (default=16)
