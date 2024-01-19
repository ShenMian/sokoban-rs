# sokoban-rs

A sokoban with a solver.

## Features

- Full mouse control.

<p align="center"><img src="./docs/auto_crate_push.gif" width=50%></p>

- Integrated solver.

<p align="center"><img src="./docs/auto_solve.gif" width=50%></p>

- Front-end and back-end separation.
- Levels and solutions are stored in the database.

## Keymap

### Mouse

| Key                                   | Action                                   |
|---------------------------------------|------------------------------------------|
| Click `Left` on player reachable area | Move the character to this position      |
| Click `Left` on a crate               | Show the pushable area of the crate      |
| Click `Left` on crate pushable area   | Push the selected crate to this position |
| Hold `Right` and drag                 | Adjust viewport                          |
| Click `Button 4`                      | Undo last push                           |
| Click `Button 5`                      | Redo last push                           |
| Scroll `Middle`                       | Zoom in/out                              |

### Keyboard

| Key                    | Action                            |
| ---------------------- | --------------------------------- |
| `W`/`A`/`S`/`D`        | Move the character                |
| Arrow keys             | Move the character                |
| `[`/`]`                | Switch to the previous/next level |
| `Ctrl` + `Z`           | Undo last push                    |
| `Ctrl` + `Shift` + `Z` | Redo last push                    |
| `Esc`                  | Reset current level               |
| `-`/`=`                | Zoom in/out                       |
| `Ctrl` + `V`           | Import levels from clipboard      |
| `Ctrl` + `C`           | Export level to clipboard         |
| `P`                    | Automatic solution                |
| `I`                    | Trigger instant move[^1]          |

### Keyboard (Vim)

| Key             | Action                            |
|-----------------|-----------------------------------|
| `H`/`J`/`K`/`L` | Move the character                |
| `U`             | Undo last push                    |
| `Ctrl` + `R`    | Redo last push                    |

### Controller

| Key               | Action                            |
|-------------------|-----------------------------------|
| D-Pad             | Move the character                |
| `LB`/`RB`         | Switch to the previous/next level |
| `B`/`Circle`      | Undo last push                    |
| `A`/`Cross`       | Redo last push                    |
| `LT`/`RT`         | Zoom in/out                       |
| Right stick       | Adjust viewport                   |
| `X`/`Square`      | Trigger instant move[^1]          |
| `Y`/`Triangle`    | Automatic solution                |

[^1]: Turn off character and crates movement animations.
