# Level management

## XSB format

| Tile             | Symbol      |
| ---------------- | ----------- |
| Wall             | `#`         |
| Player           | `@`         |
| Player on target | `+`         |
| Box              | `$`         |
| Box on target    | `*`         |
| Target           | `.`         |
| Floor            | ``/`-`/`_` |

## Import

Supports importing levels in XSB format (including run-length encoding) from files or the system clipboard:

- Import from files: Users can drag single or multiple level files in XSB format into the window.
- Import from clipboard: If levels in XSB format are already in the clipboard, they can be imported using the input action.

Levels are first standardized to prevent repeated imports of the same or similar levels. The standardized levels are used only for deduplication, and the final imported levels are not standardized.

Levels are considered similar if they meet the following conditions:

- The starting positions of the player characters are different but within the same closed area.
- After rotation and inversion.
- The difference is only in decorations that are inaccessible to the character. While these decorations may enhance the appearance of the level, they do not affect the level's solution.

## Export

Supports exporting the current level to the system clipboard in XSB format.

## Supported special level types

- [ ] Circular levels.
- [x] Levels with decorative outside elements.
- [x] Interior empty rows.
- [ ] Not closed levels.

## See also

- <http://www.sokobano.de/wiki/index.php?title=Level_format>
