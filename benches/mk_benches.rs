use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn collision_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Collision Benchmarks");
    group.bench_function("collision test - 1 step - bitvec", |b| b.iter(|| 0));
}

criterion_group!(benches, collision_bench);
criterion_main!(benches);
