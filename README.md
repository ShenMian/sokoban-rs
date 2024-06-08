# sokoban-rs

A sokoban with solver.

## Features

- **Solver**: The solver can automatically solve simple levels. [More details](docs/solver.md).

  <p align="center"><img src="./docs/assets/auto_solve.gif" width=50%></p>

- **Auto move**: Supports full mouse control for automatic player movement and crate pushing. [More details](docs/auto_move.md).

  <p align="center"><img src="./docs/assets/auto_crate_push.gif" width=50%></p>

- **Level management**: Levels and solutions are stored in the database. [More details](docs/level_management.md).
- **Customizing**: Options can be customized via configuration file. [More details](docs/customizing.md).
- **Map viewing**: Supports moving and zooming the map. When switching levels, the zoom will be automatically adjusted to fit the window size.
- **Player character animation**: Player character has smooth movement and animations in different directions.
- **Front-end and back-end separation**: Backend code (such as solvers) is decoupled from the frontend (such as Bevy).

## Acknowledgements

- anian <<anianwu@gmail.com>>: For providing comprehensive answers to numerous inquiries regarding Sokoban and offering insightful recommendations.
- [@PaperPlaneLSY](https://github.com/PaperPlaneLSY): For testing, improving the skin, and additional support.

## Keymap

### Mouse

| Key                                   | Action                                   |
| ------------------------------------- | ---------------------------------------- |
| Click `Left` on player reachable area | Move the character to this position      |
| Click `Left` on a crate               | Show the pushable area of the crate      |
| Click `Left` on crate pushable area   | Push the selected crate to this position |
| Hold `Right` and drag                 | Adjust viewport                          |
| Click `Button 4`                      | Undo the last push                       |
| Click `Button 5`                      | Redo the last push                       |
| Scroll `Middle`                       | Zoom in/out                              |

### Keyboard

| Key                       | Action                                     |
| ------------------------- | ------------------------------------------ |
| `W`/`A`/`S`/`D`           | Move the character                         |
| Arrow keys                | Move the character                         |
| `[`/`]`                   | Switch to the previous/next level          |
| `Ctrl` + `[`/`Ctrl` + `]` | Switch to the previous/next unsolved level |
| `Ctrl` + `Z`              | Undo the last push                         |
| `Ctrl` + `Shift` + `Z`    | Redo the last push                         |
| `Esc`                     | Reset current level                        |
| `-`/`=`                   | Zoom in/out                                |
| `Ctrl` + `V`              | Import levels from clipboard               |
| `Ctrl` + `C`              | Export level to clipboard                  |
| `P`                       | Toggle automatic solution                  |
| `I`                       | Toggle instant move[^1]                    |
| `F11`                     | Toggle fullscreen                          |

### Keyboard (Vim)

| Key             | Action             |
| --------------- | ------------------ |
| `H`/`J`/`K`/`L` | Move the character |
| `U`             | Undo the last push |
| `Ctrl` + `R`    | Redo the last push |

### Controller

| Key            | Action                            |
| -------------- | --------------------------------- |
| D-Pad          | Move the character                |
| `LB`/`RB`      | Switch to the previous/next level |
| `B`/`Circle`   | Undo the last push                |
| `A`/`Cross`    | Redo the last push                |
| `LT`/`RT`      | Zoom in/out                       |
| Right stick    | Adjust viewport                   |
| `X`/`Square`   | Toggle instant move[^1]           |
| `Y`/`Triangle` | Toggle automatic solution         |

[^1]: Turn off character and crates movement animations.
