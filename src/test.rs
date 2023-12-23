//extern crate test;

#[cfg(test)]
mod tests {
    use crate::level::Level;
    use std::fs;

    use super::*;
    // use test::Bencher;

    #[test]
    fn load_levels_from_file() {
        for path in fs::read_dir("assets/levels/").unwrap() {
            assert!(Level::load_from_file(&path.unwrap().path()).is_ok());
        }
    }

    // #[bench]
    // fn bench_load_levels_from_file(b: &mut Bencher) {
    //     b.iter(|| Level::load_from_file(Path::new("assets/levels/box_world.xsb")).unwrap());
    // }
}
