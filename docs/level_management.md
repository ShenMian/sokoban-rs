# Level management

## XSB format

| Tile             | Symbol      |
| ---------------- | ----------- |
| Wall             | `#`         |
| Player           | `@`         |
| Player on target | `+`         |
| Crate            | `$`         |
| Crate on target  | `*`         |
| Target           | `.`         |
| Floor            | ` `/`-`/`_` |

## Import

Supports importing levels in XSB format (support run-length encoding) from files or system clipboard:

- Import from files: The user can drag single or multiple level files in XSB format into the window.
- Import from clipboard: If some levels in XSB format is already in the clipboard, it can be imported using the input action.

The levels will first be standardized to prevent repeated import of the same or similar levels. The standardized levels are only used for deduplication, and the final imported levels are not standardized.

Levels that meet the following conditions are considered similar:

- The starting positions of the player characters are different, but in the same closed area.
- After rotation and inversion.
- The difference is only in the decorations that are inaccessible to the character. These decorations can make the level more beautiful, but do not affect the solution of the level.

## Export

Supports exporting the current level to the system clipboard in XSB format.

## Supported special level types

- [ ] Circular levels.
- [x] Levels with decorative outside elements.
- [x] Interior empty rows.
- [ ] Not closed levels.

## See also

- <http://www.sokobano.de/wiki/index.php?title=Level_format>
