# Customization

## Configuration

The user can configure settings by editing `config.toml` in the same directory. If the file does not exist, a default configuration file will be automatically generated when the program is run.

```toml
# Player movement animation speed, in seconds per step.
player_move_speed = 0.1
# Makes the floor look like a chessboard with alternating light and dark squares.
even_square_shades = 0.1
# Audio volume.
volume = 0.5
# Disables player movement animation.
instant_move = false
# Enables automatic switching to the next unsolved level when the current level is solved.
auto_switch_to_next_unsolved_level = true

[solver]
strategy = "Fast"
lower_bound_method = "MinimumMove"
```

For `solver` related configuration options, please refer to [Solver](./solver.md).

## Handling Rapid Inputs

To handle rapid and continuous inputs, the game uses an input buffer. You can configure the size of this buffer by editing the `config.toml` file.

```toml
# Input buffer size for handling rapid inputs.
input_buffer_size = 10
```

Increasing the buffer size can help in managing rapid inputs more effectively, but setting it too high may introduce input lag.
