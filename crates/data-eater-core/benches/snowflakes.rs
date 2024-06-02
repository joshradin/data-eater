use criterion::{Criterion, criterion_group, criterion_main};

use data_eater_core::snowflake::SnowflakeFactory;

fn bench_new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| SnowflakeFactory::new()));
}

fn bench_next(c: &mut Criterion) {
    let mut factory = SnowflakeFactory::new();
    c.bench_function("next", |b| b.iter(|| factory.next()));
}

fn bench_decompose(c: &mut Criterion) {
    let mut factory = SnowflakeFactory::new();
    let next_id = factory.next();
    c.bench_function("decompose", |b| b.iter(|| next_id.decompose()));
}

criterion_group!(snowflake_perf, bench_new, bench_next, bench_decompose);
criterion_main!(snowflake_perf);