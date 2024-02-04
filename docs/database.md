# Database

Any imported levels will be permanently stored in a [SQLite] database.

```
sqlite> SELECT * FROM tb_level LIMIT 1;
+----+------------+-----------------+----------+----------+-------+--------+--------------------+------------+
| id |   title    |     author      | comments |   map    | width | height |        hash        |    date    |
+----+------------+-----------------+----------+----------+-------+--------+--------------------+------------+
| 1  | Boxworld 1 | Thinking Rabbit | Level 1  |   ###    | 8     | 8      | 413779389415751139 | 2024-02-03 |
|    |            |                 |          |   #.#    |       |        |                    |            |
|    |            |                 |          |   # #### |       |        |                    |            |
|    |            |                 |          | ###$ $.# |       |        |                    |            |
|    |            |                 |          | #. $@### |       |        |                    |            |
|    |            |                 |          | ####$#   |       |        |                    |            |
|    |            |                 |          |    #.#   |       |        |                    |            |
|    |            |                 |          |    ###   |       |        |                    |            |
+----+------------+-----------------+----------+----------+-------+--------+--------------------+------------+

sqlite> SELECT * FROM tb_solution LIMIT 1;
+----------+--------------------+--------------------+
| level_id | best_move_solution | best_push_solution |
+----------+--------------------+--------------------+
| 1        | DuLLrUUdrR         | LUUddLrrDuuR       |
+----------+--------------------+--------------------+
```

[SQLite]: https://www.sqlite.org/index.html