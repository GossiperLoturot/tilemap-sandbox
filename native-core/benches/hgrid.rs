use criterion::*;
use glam::*;

use native_core::*;

pub fn benchmark_hgrid(c: &mut Criterion) {
    c.bench_function("hgrid add", |b| {
        b.iter_custom(|iters| {
            let mut hgrid = HGrid::default();

            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                let value = u16::default();
                hgrid.insert(rect, i, value);
                hgrid.remove(rect, i);
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let rect = std::hint::black_box(IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE));
                let value = std::hint::black_box(u16::default());
                std::hint::black_box(hgrid.insert(rect, i, value));
            }
            instance.elapsed()
        });
    });

    c.bench_function("hgrid remove", |b| {
        b.iter_custom(|iters| {
            let mut hgrid = HGrid::default();

            for i in 0..iters {
                let rect = IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE);
                let value = u16::default();
                hgrid.insert(rect, i, value);
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let rect = std::hint::black_box(IVec2::new(i as i32, 0) + IRect2::new(IVec2::ZERO, IVec2::ONE));
                std::hint::black_box(hgrid.remove(rect, i));
            }
            instance.elapsed()
        });
    });
}
