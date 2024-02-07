# Features

## General information

### Platforms

| Platform | Support                  |
| -------- | ------------------------ |
| Windows  | :heavy_check_mark:       |
| Linux    | :heavy_check_mark:       |
| macOS    | :heavy_check_mark:       |
| Android  | :heavy_multiplication_x: |

## Game play

### HID

| Feature    | Support                                                |
| ---------- | ------------------------------------------------------ |
| Keyboard   | :heavy_check_mark:                                     |
| Mouse      | :heavy_check_mark:                                     |
| Controller | :heavy_check_mark:                                     |
| Touchpad   | :heavy_multiplication_x: (Waiting for Android support) |

### Basic

| Feature                                                               | Support                                                             |
| --------------------------------------------------------------------- | ------------------------------------------------------------------- |
| autosave best solutions                                               | moves, pushes                                                       |
| autosave solutions options                                            | always autosaves better solutions, inferior solutions are discarded |
| push or move optimized pathfinding                                    | :heavy_check_mark:                                                  |
| move animation modes                                                  | smooth, instant                                                     |
| mouse move control: select and drop [^1]                              | :heavy_check_mark:                                                  |
| mouse move control: drag and track [^2]                               | :heavy_multiplication_x: (Waiting for Android support)              |
| reverse mode play starting at end position (pull instead of push)     | :heavy_multiplication_x:                                            |
| show secondary metrics (box-pushes, box-changes and pushing-sessions) | :heavy_multiplication_x:                                            |
| timing                                                                | :heavy_multiplication_x:                                            |
| move player through boxes                                             | :heavy_multiplication_x:                                            |

[^1]: Lift and drop. Left click on a box to lift it, move to the wanted position and left click again to drop. Lift and drop is easier for long moves than the drag and drop.
[^2]: Point at the pusher with the mouse and move it in the direction you want. The pusher will track your movements.

### Legal moves

| Feature                          | Support                  |
| -------------------------------- | ------------------------ |
| show player's reachable squares  | :heavy_check_mark:       |
| show box's reachable squares     | :heavy_check_mark:       |
| show pushable boxes              | :heavy_multiplication_x: |
| simple deadlock detection [^3]   | :heavy_multiplication_x: |
| advanced deadlock detection [^4] | :heavy_multiplication_x: |

[^3]: Dead squares/freeze deadlocks.
[^4]: Bipartite deadlocks/corral deadlocks.

### History

| Feature               | Support                  |
| --------------------- | ------------------------ |
| undo/redo             | unlimited                |
| undo all              | :heavy_multiplication_x: |
| redo all              | :heavy_multiplication_x: |
| replay                | :heavy_multiplication_x: |
| remember last session | :heavy_multiplication_x: |

## Customizing

| Feature            | Support          |
| ------------------ | ---------------- |
| settings interface | TOML config file |

## Skin features

| Feature                    | Support                  |
| -------------------------- | ------------------------ |
| customizable skin          | :heavy_check_mark:       |
| resizeable skins           | :heavy_multiplication_x: |
| directional player         | :heavy_check_mark:       |
| support for seamless walls | :heavy_multiplication_x: |
| walls can be transparent   | :heavy_check_mark:       |
| floors can be transparent  | :heavy_check_mark:       |

## Map viewing

| Feature                              | Support                  |
| ------------------------------------ | ------------------------ |
| resize skin to fit window            | :heavy_check_mark:       |
| levels larger than window: scrolling | :heavy_check_mark:       |
| search for unsolved level            | next, previous           |
| rotate and mirror map                | :heavy_multiplication_x: |
| full screen                          | :heavy_check_mark:       |

## Level management

### Basic

| Feature                               | Support                  |
| ------------------------------------- | ------------------------ |
| copy/paste levels                     | :heavy_check_mark:       |
| copy/paste levels: run-length-encoded | :heavy_check_mark:       |
| copy levels with solutions            | :heavy_multiplication_x: |
| paste levels with solutions           | :heavy_multiplication_x: |
| load multiple files                   | :heavy_check_mark:       |
| saves levels with format              | SQLite database          |

### Import and export for Internet

| Feature                                           | Support                  |
| ------------------------------------------------- | ------------------------ |
| import levels with hyphens "-" or underscores "_" | :heavy_check_mark:       |
| export levels with hyphens "-" or underscores "_" | :heavy_multiplication_x: |

## Features will not be implemented

| Feature                    | Reasons                                                   |
| -------------------------- | --------------------------------------------------------- |
| level browser with preview | It is not appropriate to use the Bevy engine to implement |
| Support iOS                | No device running iOS                                     |

## See also

- <http://sokobano.de/wiki/index.php?title=Feature_list>
