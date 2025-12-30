//! Benchmark tests for virtual_audio audio processing.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use virtual_audio_cable::{AudioFormat, AudioProcessor, RingBuffer, TripleRingBuffer};

fn benchmark_ring_buffer_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_write");

    for size in [256, 512, 1024, 2048, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut buffer = RingBuffer::<f32>::new(size);
            let data = vec![1.0f32; size / 2];

            b.iter(|| {
                buffer.write(&data);
            });
        });
    }

    group.finish();
}

fn benchmark_ring_buffer_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_read");

    for size in [256, 512, 1024, 2048, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut buffer = RingBuffer::<f32>::new(size);
            let data = vec![1.0f32; size];
            buffer.write(&data);

            let mut output = vec![0.0f32; size];

            b.iter(|| {
                buffer.read(&mut output);
            });
        });
    }

    group.finish();
}

fn benchmark_triple_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("triple_buffer");

    for size in [256, 512, 1024, 2048].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut triple = TripleRingBuffer::new(size);
            let input = vec![1.0f32; size];
            let mut output = vec![0.0f32; size];

            b.iter(|| {
                let _ = triple.process(&input, &mut output);
            });
        });
    }

    group.finish();
}

fn benchmark_format_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_conversion");

    let formats = vec![
        ("F32LE", AudioFormat::F32LE),
        ("S16LE", AudioFormat::S16LE),
        ("S24LE", AudioFormat::S24LE),
        ("S32LE", AudioFormat::S32LE),
    ];

    let sizes = [256, 512, 1024, 2048];

    for (name, format) in formats {
        for &size in &sizes {
            group.bench_with_input(BenchmarkId::new(name, size), &size, |b, &size| {
                let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
                let input = vec![0.5f32; size];

                b.iter(|| {
                    processor.convert_format(black_box(&input), format);
                });
            });
        }
    }

    group.finish();
}

fn benchmark_format_conversion_back(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_conversion_back");

    let formats = vec![
        ("F32LE", AudioFormat::F32LE),
        ("S16LE", AudioFormat::S16LE),
        ("S24LE", AudioFormat::S24LE),
        ("S32LE", AudioFormat::S32LE),
    ];

    let sizes = [256, 512, 1024, 2048];

    for (name, format) in formats {
        for &size in &sizes {
            group.bench_with_input(BenchmarkId::new(name, size), &size, |b, &size| {
                let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
                let input = vec![0.5f32; size];
                let bytes = processor.convert_format(&input, format);

                b.iter(|| {
                    processor.bytes_to_samples(black_box(&bytes), format);
                });
            });
        }
    }

    group.finish();
}

fn benchmark_audio_processor_passthrough(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_processor_passthrough");

    for size in [256, 512, 1024, 2048, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let processor = AudioProcessor::new(48000, 48000, 2, AudioFormat::F32LE);
            let input = vec![0.5f32; size];
            let mut output = vec![0.0f32; size];

            b.iter(|| {
                processor.process(&input, &mut output).unwrap();
            });
        });
    }

    group.finish();
}

fn benchmark_resampling_up(c: &mut Criterion) {
    let mut group = c.benchmark_group("resampling_up");

    let input_sizes = [256, 512, 1024, 2048];
    let ratios = [("2x", 2.0), ("1.5x", 1.5)];

    for (ratio_name, ratio) in ratios {
        for &size in &input_sizes {
            group.bench_with_input(BenchmarkId::new(ratio_name, size), &size, |b, &size| {
                let resampler =
                    AudioProcessor::new(48000, (48000.0 * ratio) as u32, 2, AudioFormat::F32LE);
                let input = vec![0.5f32; size];
                let mut output = vec![0.0f32; (size as f64 * ratio) as usize];

                b.iter(|| {
                    resampler.process(&input, &mut output).unwrap();
                });
            });
        }
    }

    group.finish();
}

fn benchmark_resampling_down(c: &mut Criterion) {
    let mut group = c.benchmark_group("resampling_down");

    let input_sizes = [256, 512, 1024, 2048];
    let ratios = [("0.5x", 0.5), ("0.75x", 0.75)];

    for (ratio_name, ratio) in ratios {
        for &size in &input_sizes {
            group.bench_with_input(BenchmarkId::new(ratio_name, size), &size, |b, &size| {
                let resampler =
                    AudioProcessor::new(48000, (48000.0 * ratio) as u32, 2, AudioFormat::F32LE);
                let input = vec![0.5f32; size];
                let mut output = vec![0.0f32; (size as f64 * ratio) as usize];

                b.iter(|| {
                    resampler.process(&input, &mut output).unwrap();
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ring_buffer_write,
    benchmark_ring_buffer_read,
    benchmark_triple_buffer,
    benchmark_format_conversion,
    benchmark_format_conversion_back,
    benchmark_audio_processor_passthrough,
    benchmark_resampling_up,
    benchmark_resampling_down,
);
criterion_main!(benches);
