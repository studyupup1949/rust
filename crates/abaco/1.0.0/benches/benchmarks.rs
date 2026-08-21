use criterion::{Criterion, black_box, criterion_group, criterion_main};

use abaco::{Evaluator, UnitRegistry, dsp};

fn bench_eval_simple(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_simple");

    group.bench_function("addition", |b| b.iter(|| eval.eval(black_box("2 + 3"))));
    group.bench_function("mixed_ops", |b| {
        b.iter(|| eval.eval(black_box("2 + 3 * 4")))
    });
    group.bench_function("parentheses", |b| {
        b.iter(|| eval.eval(black_box("(2 + 3) * 4")))
    });
    group.bench_function("division", |b| b.iter(|| eval.eval(black_box("10 / 3"))));

    group.finish();
}

fn bench_eval_functions(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_functions");

    group.bench_function("sqrt", |b| b.iter(|| eval.eval(black_box("sqrt(16)"))));
    group.bench_function("sin", |b| b.iter(|| eval.eval(black_box("sin(3.14159)"))));
    group.bench_function("log2", |b| b.iter(|| eval.eval(black_box("log2(1024)"))));
    group.bench_function("pow", |b| b.iter(|| eval.eval(black_box("pow(2, 10)"))));
    group.bench_function("min", |b| b.iter(|| eval.eval(black_box("min(3, 5)"))));

    group.finish();
}

fn bench_eval_complex(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_complex");

    group.bench_function("mixed_ops_funcs", |b| {
        b.iter(|| eval.eval(black_box("2^3 + sqrt(9) * 2")))
    });
    group.bench_function("nested_parens", |b| {
        b.iter(|| eval.eval(black_box("((1 + 2) * (3 + 4)) / (5 - 1)")))
    });
    group.bench_function("trig_chain", |b| {
        b.iter(|| eval.eval(black_box("sin(pi / 4) * cos(pi / 4)")))
    });
    group.bench_function("long_addition", |b| {
        b.iter(|| eval.eval(black_box("1+2+3+4+5+6+7+8+9+10")))
    });

    group.finish();
}

fn bench_eval_scientific(c: &mut Criterion) {
    let eval = Evaluator::new();
    let mut group = c.benchmark_group("eval_scientific");

    group.bench_function("sci_add", |b| {
        b.iter(|| eval.eval(black_box("1.5e10 + 2.3e9")))
    });
    group.bench_function("sci_mul", |b| b.iter(|| eval.eval(black_box("1e-3 * 1e3"))));

    group.finish();
}

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");

    group.bench_function("simple", |b| {
        b.iter(|| abaco::tokenize(black_box("2 + 3 * 4")))
    });
    group.bench_function("complex", |b| {
        b.iter(|| abaco::tokenize(black_box("sin(pi / 4) * cos(pi / 4) + sqrt(abs(-16)) ^ 2")))
    });

    group.finish();
}

fn bench_unit_conversion(c: &mut Criterion) {
    let reg = UnitRegistry::new();
    let mut group = c.benchmark_group("unit_conversion");

    group.bench_function("km_to_miles", |b| {
        b.iter(|| reg.convert(black_box(100.0), "km", "mi"))
    });
    group.bench_function("celsius_to_fahrenheit", |b| {
        b.iter(|| reg.convert(black_box(100.0), "celsius", "fahrenheit"))
    });
    group.bench_function("bytes_to_gb", |b| {
        b.iter(|| reg.convert(black_box(1_000_000_000.0), "byte", "GB"))
    });
    group.bench_function("bytes_to_gib", |b| {
        b.iter(|| reg.convert(black_box(1_073_741_824.0), "byte", "GiB"))
    });
    group.bench_function("gb_to_gib_cross", |b| {
        b.iter(|| reg.convert(black_box(1.0), "GB", "GiB"))
    });
    group.bench_function("same_unit_identity", |b| {
        b.iter(|| reg.convert(black_box(42.0), "km", "km"))
    });

    group.finish();
}

fn bench_unit_lookup(c: &mut Criterion) {
    let reg = UnitRegistry::new();
    let mut group = c.benchmark_group("unit_lookup");

    group.bench_function("exact_symbol", |b| {
        b.iter(|| reg.find_unit(black_box("km")))
    });
    group.bench_function("by_name", |b| {
        b.iter(|| reg.find_unit(black_box("kilometer")))
    });
    group.bench_function("case_insensitive", |b| {
        b.iter(|| reg.find_unit(black_box("Kilometer")))
    });
    group.bench_function("plural", |b| b.iter(|| reg.find_unit(black_box("meters"))));
    group.bench_function("miss", |b| {
        b.iter(|| reg.find_unit(black_box("nonexistent")))
    });
    group.bench_function("alias_kph", |b| b.iter(|| reg.find_unit(black_box("kph"))));
    group.bench_function("alias_degree_symbol", |b| {
        b.iter(|| reg.find_unit(black_box("°C")))
    });

    group.finish();
}

fn bench_unit_conversion_extended(c: &mut Criterion) {
    let reg = UnitRegistry::new();
    let mut group = c.benchmark_group("unit_conversion_ext");

    group.bench_function("mpg_to_l100km", |b| {
        b.iter(|| reg.convert(black_box(30.0), "mpg", "L/100km"))
    });
    group.bench_function("density_g_cm3_to_kg_m3", |b| {
        b.iter(|| reg.convert(black_box(1.0), "g/cm3", "kg/m3"))
    });
    group.bench_function("viscosity_cp_to_pas", |b| {
        b.iter(|| reg.convert(black_box(1.0), "cP", "Pa·s"))
    });
    group.bench_function("lux_to_fc", |b| {
        b.iter(|| reg.convert(black_box(100.0), "lx", "fc"))
    });

    group.finish();
}

fn bench_registry_creation(c: &mut Criterion) {
    c.bench_function("registry_creation", |b| b.iter(UnitRegistry::new));
}

fn bench_dsp_db(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_db");

    group.bench_function("amplitude_to_db", |b| {
        b.iter(|| dsp::amplitude_to_db(black_box(0.5)))
    });
    group.bench_function("db_to_amplitude", |b| {
        b.iter(|| dsp::db_to_amplitude(black_box(-6.0)))
    });
    group.bench_function("amplitude_to_db_f64", |b| {
        b.iter(|| dsp::amplitude_to_db_f64(black_box(0.5)))
    });
    group.bench_function("db_to_amplitude_f64", |b| {
        b.iter(|| dsp::db_to_amplitude_f64(black_box(-6.0)))
    });
    group.bench_function("db_gain_factor", |b| {
        b.iter(|| dsp::db_gain_factor(black_box(12.0)))
    });

    group.finish();
}

fn bench_dsp_midi(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_midi");

    group.bench_function("midi_to_freq", |b| {
        b.iter(|| dsp::midi_to_freq(black_box(69.0)))
    });
    group.bench_function("freq_to_midi", |b| {
        b.iter(|| dsp::freq_to_midi(black_box(440.0)))
    });

    group.finish();
}

fn bench_dsp_envelope(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_envelope");

    group.bench_function("time_constant_10ms", |b| {
        b.iter(|| dsp::time_constant(black_box(10.0), black_box(44100)))
    });
    group.bench_function("time_constant_100ms", |b| {
        b.iter(|| dsp::time_constant(black_box(100.0), black_box(44100)))
    });

    group.finish();
}

fn bench_dsp_waveform(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_waveform");

    group.bench_function("poly_blep_mid", |b| {
        b.iter(|| dsp::poly_blep(black_box(0.5), black_box(0.01)))
    });
    group.bench_function("poly_blep_near_edge", |b| {
        b.iter(|| dsp::poly_blep(black_box(0.005), black_box(0.01)))
    });
    group.bench_function("angular_frequency", |b| {
        b.iter(|| dsp::angular_frequency(black_box(1000.0), black_box(44100.0)))
    });

    group.finish();
}

fn bench_dsp_pan(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_pan");

    group.bench_function("constant_power_pan_center", |b| {
        b.iter(|| dsp::constant_power_pan(black_box(0.0)))
    });
    group.bench_function("constant_power_pan_left", |b| {
        b.iter(|| dsp::constant_power_pan(black_box(-1.0)))
    });
    group.bench_function("equal_power_crossfade_mid", |b| {
        b.iter(|| dsp::equal_power_crossfade(black_box(0.5)))
    });

    group.finish();
}

fn bench_dsp_sanitize(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsp_sanitize");

    group.bench_function("sanitize_finite", |b| {
        b.iter(|| dsp::sanitize_sample(black_box(0.5)))
    });
    group.bench_function("sanitize_nan", |b| {
        b.iter(|| dsp::sanitize_sample(black_box(f32::NAN)))
    });

    group.finish();
}

/// Batch benchmark: process 4096 samples through common DSP functions.
fn bench_dsp_batch(c: &mut Criterion) {
    let samples: Vec<f32> = (0..4096).map(|i| (i as f32 / 4096.0) * 2.0 - 1.0).collect();
    let mut group = c.benchmark_group("dsp_batch_4096");

    group.bench_function("amplitude_to_db", |b| {
        b.iter(|| {
            let s = black_box(&samples);
            s.iter()
                .map(|&v| dsp::amplitude_to_db(v.abs().max(1e-10)))
                .sum::<f32>()
        })
    });
    group.bench_function("db_to_amplitude", |b| {
        b.iter(|| {
            let s = black_box(&samples);
            s.iter()
                .map(|&v| dsp::db_to_amplitude(v * 60.0))
                .sum::<f32>()
        })
    });
    group.bench_function("sanitize_sample", |b| {
        b.iter(|| {
            let s = black_box(&samples);
            s.iter().map(|&v| dsp::sanitize_sample(v)).sum::<f32>()
        })
    });
    group.bench_function("poly_blep", |b| {
        b.iter(|| {
            let s = black_box(&samples);
            s.iter()
                .map(|&v| dsp::poly_blep((v.abs()) as f64, 0.01) as f32)
                .sum::<f32>()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_eval_simple,
    bench_eval_functions,
    bench_eval_complex,
    bench_eval_scientific,
    bench_tokenizer,
    bench_unit_conversion,
    bench_unit_conversion_extended,
    bench_unit_lookup,
    bench_registry_creation,
    bench_dsp_db,
    bench_dsp_midi,
    bench_dsp_envelope,
    bench_dsp_waveform,
    bench_dsp_pan,
    bench_dsp_sanitize,
    bench_dsp_batch,
);
criterion_main!(benches);
