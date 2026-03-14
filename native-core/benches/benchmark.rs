mod tile;
mod block;
mod entity;
mod hgrid;

use criterion::*;

criterion_group!(benches, tile::benchmark_tile, block::benchmark_block, entity::benchmark_entity, hgrid::benchmark_hgrid);
criterion_main!(benches);
