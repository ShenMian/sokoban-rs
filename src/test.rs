// extern crate test;

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::level::Level;
    use crate::solver::solver::*;
    use std::fs;
    use std::path::Path;

    // use test::Bencher;

    #[test]
    fn load_levels_from_file() {
        for path in fs::read_dir("assets/levels/").unwrap() {
            assert!(Level::load_from_file(&path.unwrap().path()).is_ok());
        }
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn solver() {
        let levels = Level::load_from_file(Path::new("assets/levels/box_world.xsb")).unwrap();
        let mut failed = 0;
        for i in [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 21, 24, 28, 30, 31,
            34, 36, 40, 43, 61, 62, 64, 65, 69, 74, 76, 90, 91,
        ] {
            let level = levels[i].clone();
            let mut solver = Solver::new(level.clone());
            solver.initial(Strategy::Fast, LowerBoundMethod::PushCount);
            let solution = solver.solve(std::time::Duration::from_secs(30));
            if solution.is_err() {
                println!("#{}", i + 1);
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

        assert!(failed == 0);
    }

    // #[bench]
    // fn bench_load_levels_from_file(b: &mut Bencher) {
    //     b.iter(|| Level::load_from_file(Path::new("assets/levels/box_world.xsb")).unwrap());
    // }
}
