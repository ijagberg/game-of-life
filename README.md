# game-of-life

Rust implementation of Conway's Game of Life, using [ggez](https://github.com/ggez/ggez)

![alt text](https://github.com/ijagberg/game-of-life/raw/master/example.gif "Logo Title Text 1")

# Controls

* Click anywhere to create a living cell at that location, or to kill a living cell. 
* Click while holding shift to move the camera around.
* Scroll out/in to change zoom level.
* Space to pause simulation.
* D to step forward 1 generation while paused.

# Parameters

* --debug, enable debug logging
* --file <FILE>, sets the initial state of the game to the state in <FILE> (supports "txt" and "rle" extensions)
  ```game-of-life --file resources/koks_galaxy.txt```
* --updates-per-second <FLOAT>, sets the refresh rate of the game, in order to speed up/slow down simulation (default=16.0)
  ```game-of-life --updates-per-second 32```
