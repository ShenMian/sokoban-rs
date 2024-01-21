# Solver

<p align="center"><img src="auto_solve.gif" width=70%></p>

The solver can automatically solve simple levels.

## Strategy

- `Fast`: Speed priority.
- `Mixed`: Balanced speed and steps.
- `OptimalMovePush`: Find move optimal solutions with best pushes.
- `OptimalPushMove`: Find push optimal solutions with best moves.

## Lower bound calculation method

- `PushCount`: Minimum push count to nearest target.
- `MoveCount`: Minimum move count to nearest target.
- `ManhattanDistance`: Manhattan distance to nearest target.

## Optimization

- Deadlocks detection.
  - Dead square deadlocks.
  - Freeze deadlocks.
- Tunnels detection.

## Visualization

Supports visualization of the automatic solution process. This feature can be used to intuitively view the working status of the solver.

- Displays the optimal state obtained by the current solver.
- Display the lower bound as a heat map.

![Visualization](solver_visualization.png)
