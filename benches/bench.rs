use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

pub fn criterion_benchmark(c: &mut Criterion) {
    for size in [0, 1 << 4, 1 << 8, 1 << 12, 1 << 16, 1 << 20, 1 << 24] {
        let mut group = c.benchmark_group(&format!("{size:#x}"));
        group.throughput(Throughput::Bytes(size as u64));
        let mut vec = vec![0; size];
        black_box(&vec);

        group.bench_function("memset_nt", |b| {
            b.iter(|| {
                memset_nt::memset(&mut vec, 2);
                black_box(&vec.get(4));
            });
        });
        group.bench_function("write_bytes", |b| {
            b.iter(|| unsafe {
                std::ptr::write_bytes(vec.as_mut_ptr(), 2, vec.len());
                black_box(&vec.get(4));
            });
        });
        black_box(&vec);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
