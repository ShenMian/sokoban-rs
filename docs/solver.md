# Solver

<p align="center"><img src="assets/auto_solve.gif" width=70%></p>

The solver can automatically solve levels of moderate complexity.

## Strategy

- `Fast`: Prioritizes speed.
- `Mixed`: Balances speed and steps.
- `OptimalMovePush`: Finds optimal move solutions with the fewest pushes.
- `OptimalPushMove`: Finds optimal push solutions with the fewest moves.

## Lower bound calculation method

- `MinimumPush`: Counts the minimum number of pushes to the nearest target.
- `MinimumMove`: Counts the minimum number of moves to the nearest target. (This method is slow, especially on maps with many boxes or large areas)
- `ManhattanDistance`: Uses Manhattan distance to the nearest target. (This method is fast and suitable for maps with many boxes or large areas)

## Optimization

- Deadlocks detection.
  - Detects dead square deadlocks.
  - Detects freeze deadlocks.
- Tunnels detection.

## Statistics

CPU       : 13th Gen Intel(R) Core(TM) i9-13900HX (Base speed: 2.20 GHz).  
Threads   : 1.  
Time limit: 10 seconds.

| Collection          | Total | Solved |
| ------------------- | ----- | ------ |
| box_world_100.xsb   | 100   | 41     |
| microban_155.xsb    | 155   | 144    |
| microban_II_135.xsb | 135   | 109    |

## Visualization

Supports visualizing the automatic solution process. This feature allows you to intuitively view the working status of the solver.

- Displays the best state found by the solver.
- Displays lower bounds as a heat map.

<p align="center"><img src="assets/solver_visualization.png" width=70%></p>
