mod tile;
mod block;
mod entity;
mod paging;

use criterion::*;

criterion_group!(benches, tile::benchmark_tile, block::benchmark_block, entity::benchmark_entity, paging::benchmark_paging);
criterion_main!(benches);
