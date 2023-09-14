use criterion::{criterion_group, criterion_main, Criterion};
use key_value_parser::zero_copy;

fn keyvalue_fullcopy(data: &str) {
    let _parser = key_value_parser::full_copy::Parser::new(data).unwrap();
}

fn keyvalue_zerocopy(data: &str) {
    let _parser = key_value_parser::zero_copy::Parser::new(data).unwrap();
}

fn keyvalue_almost_zerocopy(data: &str) {
    let _parser = key_value_parser::almost_zero_copy::Parser::new(data).unwrap();
}

fn keyvalue_full_almost_zerocopy(data: &str) {
    let _parser = key_value_parser::full_almost_zero_copy::Parser::new(data).unwrap();
}

fn keyvalue_zero_parse(data: &str, keys: &[String]) {
    for k in keys {
        key_value_parser::zero_parse::parse(data, k).unwrap();
    }
}

fn criterion_benchmark_nonquote(c: &mut Criterion) {
    // create test data.  1000 key/value pairs
    let mut data = String::new();
    for i in 0..1000 {
        data.push_str(&format!("key{}=value{} ", i, i));
    }

    c.bench_function("keyvalue_fullcopy", |b| b.iter(|| keyvalue_fullcopy(&data)));
    c.bench_function("keyvalue_zerocopy", |b| b.iter(|| keyvalue_zerocopy(&data)));
    c.bench_function("keyvalue_almost_zerocopy", |b| {
        b.iter(|| keyvalue_almost_zerocopy(&data))
    });

    // create test data.  1000 key/value pairs where each key and value is 1000 characters long
    let mut data = String::new();
    for i in 0..1000 {
        data.push_str(&format!("{}{i}={}{i}", "k".repeat(1000), "v".repeat(1000)));
    }

    c.bench_function("keyvalue_fullcopy_1000_1000", |b| {
        b.iter(|| keyvalue_fullcopy(&data))
    });
    c.bench_function("keyvalue_zerocopy_1000_1000", |b| {
        b.iter(|| keyvalue_zerocopy(&data))
    });
    c.bench_function("keyvalue_almost_zerocopy_1000_1000", |b| {
        b.iter(|| keyvalue_almost_zerocopy(&data))
    });
}

fn criterion_benchmark_quote(c: &mut Criterion) {
    // create test data.  1000 key/value pairs
    let mut data = String::new();
    let mut keys = Vec::new();
    for i in 0..1000 {
        let key = format!("key{}", i);
        keys.push(key.clone());
        data.push_str(&format!("{}=\"value{}\" ", key, i));
    }

    c.bench_function("keyvalue_fullcopy", |b| b.iter(|| keyvalue_fullcopy(&data)));
    c.bench_function("keyvalue_full_almost_zerocopy", |b| {
        b.iter(|| keyvalue_full_almost_zerocopy(&data))
    });
    c.bench_function("keyvalue_zero_parse", |b| {
        b.iter(|| keyvalue_zero_parse(&data,&keys))
    });

    // create test data.  1000 key/value pairs where each key and value is 1000 characters long
    let mut data = String::new();
    let mut keys = Vec::new();

    for i in 0..1000 {
        let key = format!("{}{i}", "k".repeat(1000));
        keys.push(key.clone());
        data.push_str(&format!(
            "{}=\"{}{i}\" ",
            key,
            "v".repeat(1000)
        ));
    }

    c.bench_function("keyvalue_fullcopy_1000_1000", |b| {
        b.iter(|| keyvalue_fullcopy(&data))
    });
    c.bench_function("keyvalue_full_almost_zerocopy_1000_1000", |b| {
        b.iter(|| keyvalue_full_almost_zerocopy(&data))
    });
    c.bench_function("keyvalue_zero_parse_1000_1000", |b| {
        b.iter(|| keyvalue_zero_parse(&data,&keys))
    });

    // create test data.  1000 key/value pairs where each key and value is 1000 characters long with escape characters
    let mut data = String::new();
    let mut keys = Vec::new();
    for i in 0..1000 {
        let key = format!("{}{i}", "k".repeat(1000));
        keys.push(key.clone());
        data.push_str(&format!(
            "{}=\"{}\\\"{}{i}\" ",
            key,
            "v".repeat(500),
            "t".repeat(500)
        ));
    }

    c.bench_function("keyvalue_fullcopy_1000_1000_escaped", |b| {
        b.iter(|| keyvalue_fullcopy(&data))
    });
    c.bench_function("keyvalue_full_almost_zerocopy_1000_1000_escaped", |b| {
        b.iter(|| keyvalue_full_almost_zerocopy(&data))
    });
    c.bench_function("keyvalue_zero_parse_1000_1000_escaped", |b| {
        b.iter(|| keyvalue_zero_parse(&data,&keys))
    });
}

criterion_group!(
    benches,
    //criterion_benchmark_nonquote,
    criterion_benchmark_quote
);
criterion_main!(benches);
