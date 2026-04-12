use criterion::*;
use glam::*;

use native_core::*;

pub fn benchmark_paging(c: &mut Criterion) {
    c.bench_function("paging grid add", |b| {
        b.iter_custom(|iters| {
            let mut grid = SpatialIndex::default();

            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                grid.insert(rect, i);
                grid.remove(rect);
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                grid.insert(std::hint::black_box(rect), std::hint::black_box(i));
            }
            instance.elapsed()
        });
    });

    c.bench_function("paging grid remove", |b| {
        b.iter_custom(|iters| {
            let mut grid = SpatialIndex::default();

            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                grid.insert(rect, i);
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                std::hint::black_box(grid.remove(std::hint::black_box(rect)));
            }
            instance.elapsed()
        });
    });
}
