// extern crate test;

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::level::Level;
    use crate::solver::solver::*;
    use std::fs;
    use std::ops::RangeBounds;
    use std::path::Path;
    use std::time::Duration;

    // use test::Bencher;

    #[test]
    fn load_levels_from_file() {
        for path in fs::read_dir("assets/levels/").unwrap() {
            assert!(Level::load_from_file(&path.unwrap().path()).is_ok());
        }
    }

    fn solve<R: RangeBounds<usize> + IntoIterator<Item = usize>>(
        levels: &[Level],
        range: R,
        exclude: &[usize],
        time_limit: u64,
    ) -> i32 {
        let mut failed = 0;
        for id in range {
            if exclude.contains(&id) {
                continue;
            }
            println!("#{} ({})", id + 1, id);
            let level = levels[id].clone();
            let mut solver = Solver::new(level.clone());
            solver.initial(Strategy::Fast, LowerBoundMethod::MinimumMove);
            let solution = solver.solve(Duration::from_secs(time_limit));
            if solution.is_err() {
                println!("{}", level.export_map());
                println!("{:?}\n\n", solution.clone().err());
                failed += 1;
                continue;
            }
            let solution = solution.unwrap();

            let mut board = Board::with_level(level);
            for movement in &*solution {
                board.move_or_push(movement.direction);
            }
            assert!(board.is_solved());
        }
        failed
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn solver_microban() {
        let levels = Level::load_from_file(Path::new("assets/levels/microban_155.xsb")).unwrap();
        assert!(
            solve(
                &levels,
                0..155,
                &[
                    35, 92, 97, 107, 108, 110, 113, 116, 121, 137, 138, 142, 144, 145, 149, 150,
                    152, 154
                ],
                15 * 2
            ) == 0
        );
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn solver_box_world() {
        let levels = Level::load_from_file(Path::new("assets/levels/box_world_100.xsb")).unwrap();
        assert!(
            solve(
                &levels,
                0..100,
                &[
                    15, 17, 20, 22, 23, 25, 26, 27, 32, 33, 35, 37, 38, 39, 41, 42, 44, 45, 46, 47,
                    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 60, 66, 67, 68, 70, 71, 72, 73, 75,
                    78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 92, 93, 94, 95, 96, 97, 98, 99
                ],
                15 * 2
            ) == 0
        );
    }

    // #[bench]
    // fn bench_load_levels_from_file(b: &mut Bencher) {
    //     b.iter(|| Level::load_from_file(Path::new("assets/levels/box_world.xsb")).unwrap());
    // }
}
