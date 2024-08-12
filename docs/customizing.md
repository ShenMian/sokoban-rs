# Customizing

## Configuration

The user can configure settings by editing `config.toml` in the same directory. If the file does not exist, a default configuration file will be automatically generated when the program is run.

```toml
# Player movement animation speed, seconds per step.
player_move_speed = 0.1
# Make the floor look like a chessboard with alternating light square and dark square.
even_square_shades = 0.1
# Audio volume.
volume = 0.5
# Disable player movement animation.
instant_move = false
# Enable auto switch to next unsolved level when the current level is solved.
auto_switch_to_next_unsolved_level = true

[solver]
strategy = "Fast"
lower_bound_method = "MinimumMove"
```

For `solver` related configuration options, please refer to [Solver](./solver.md).
