# sokoban-rs

A simple sokoban.

## Keymap

### Keyboard

| Key             | Action                            |
|-----------------|-----------------------------------|
| `W`/`A`/`S`/`D` | Move the character                |
| Arrow keys      | Move the character                |
| `[`/`]`         | Switch to the previous/next level |
| `Ctrl` + `Z`    | Undo last push                    |
| `-`/`=`         | Zoom in/out                       |
| `Ctrl` + `V`    | Import levels from clipboard      |
| `Ctrl` + `C`    | Export level to clipboard         |

### Keyboard (Vim)

| Key             | Action                            |
|-----------------|-----------------------------------|
| `H`/`J`/`K`/`L` | Move the character                |
| `[`/`]`         | Switch to the previous/next level |
| `U`             | Undo last push                    |

### Controller

| Key               | Action                            |
|-------------------|-----------------------------------|
| D-Pad             | Move the character                |
| `LB`[^1]/`RB`[^2] | Switch to the previous/next level |
| `B` / `Circle`    | Undo last push                    |

### Mouse

| Key                                   | Action                              |
|---------------------------------------|-------------------------------------|
| Click `Left` on player reachable area | Move the character to this position |
| Hold `Right` and drag                 | Drag the board                      |
| Scroll `Middle`                       | Zoom in/out                         |

[^1]: Left Bumper.
[^2]: Right Bumper.
