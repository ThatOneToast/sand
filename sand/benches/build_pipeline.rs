use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use sand::build::package::zip_dir;
use sand::build::records::{ComponentRecord, ResourcePackRecord};
use sand::build::validate::{validate_component_records, validate_resourcepack_records};
use sand::build::write::{write_component, write_rp_record};
use sand::run_cmd;

fn component_records(count: usize) -> Vec<ComponentRecord> {
    let records = (0..count)
        .map(|i| {
            serde_json::json!({
                "namespace": "bench",
                "dir": "function",
                "path": format!("generated/fn_{i}"),
                "ext": "mcfunction",
                "content": format!("say function {i}\n"),
            })
        })
        .collect::<Vec<_>>();

    serde_json::from_value(serde_json::Value::Array(records)).unwrap()
}

fn resource_records(count: usize) -> Vec<ResourcePackRecord> {
    let records = (0..count)
        .map(|i| {
            serde_json::json!({
                "path": format!("assets/bench/models/item/item_{i}.json"),
                "content_type": "json",
                "content": format!(r#"{{"parent":"minecraft:item/generated","bench":{i}}}"#),
            })
        })
        .collect::<Vec<_>>();

    serde_json::from_value(serde_json::Value::Array(records)).unwrap()
}

fn bench_validation(c: &mut Criterion) {
    let records = component_records(128);
    let dist = std::path::Path::new("dist/bench");

    c.bench_function("record validation/128 functions", |b| {
        b.iter(|| validate_component_records(dist, criterion::black_box(&records)).unwrap());
    });
}

fn bench_generated_json_parsing(c: &mut Criterion) {
    let records = component_records(128);
    let json = serde_json::to_vec(
        &(0..records.len())
            .map(|i| {
                serde_json::json!({
                    "namespace": "bench",
                    "dir": "recipe",
                    "path": format!("recipe_{i}"),
                    "ext": "json",
                    "content": r#"{"type":"minecraft:crafting_shapeless","ingredients":[],"result":{"id":"minecraft:stone"}}"#,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();

    c.bench_function("generated JSON parse/128 records", |b| {
        b.iter(|| {
            let parsed: Vec<ComponentRecord> = serde_json::from_slice(criterion::black_box(&json))
                .expect("benchmark fixture parses");
            criterion::black_box(parsed);
        });
    });
}

fn bench_component_writing(c: &mut Criterion) {
    let records = component_records(128);

    c.bench_function("component writing/128 functions", |b| {
        b.iter_batched(
            || tempfile::tempdir().unwrap(),
            |temp| {
                let dist = temp.path().join("bench");
                for record in &records {
                    write_component(&dist, temp.path(), record).unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_resource_record_writing(c: &mut Criterion) {
    let records = resource_records(64);

    c.bench_function("resource-pack writing/64 json assets", |b| {
        b.iter_batched(
            || tempfile::tempdir().unwrap(),
            |temp| {
                validate_resourcepack_records(&records).unwrap();
                let dist = temp.path().join("bench-resources");
                for record in &records {
                    write_rp_record(&dist, temp.path(), record).unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_zip_packaging(c: &mut Criterion) {
    c.bench_function("zip packaging/128 files", |b| {
        b.iter_batched(
            || {
                let temp = tempfile::tempdir().unwrap();
                let dist = temp.path().join("bench");
                for i in 0..128 {
                    let path =
                        dist.join(format!("data/bench/function/generated/fn_{i}.mcfunction"));
                    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                    std::fs::write(path, format!("say function {i}\n")).unwrap();
                }
                temp
            },
            |temp| {
                let zip = zip_dir(&temp.path().join("bench"), "bench").unwrap();
                criterion::black_box(zip);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_server_sync(c: &mut Criterion) {
    c.bench_function("server sync/skip unchanged and prune stale", |b| {
        b.iter_batched(
            || {
                let temp = tempfile::tempdir().unwrap();
                let src = temp.path().join("dist/bench");
                let dest = temp.path().join("server/world/datapacks/bench");
                for i in 0..128 {
                    let rel = format!("data/bench/function/generated/fn_{i}.mcfunction");
                    let src_path = src.join(&rel);
                    let dest_path = dest.join(&rel);
                    std::fs::create_dir_all(src_path.parent().unwrap()).unwrap();
                    std::fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
                    let content = format!("say function {i}\n");
                    std::fs::write(src_path, &content).unwrap();
                    std::fs::write(dest_path, &content).unwrap();
                }
                let stale = dest.join("data/bench/function/generated/stale.mcfunction");
                std::fs::write(stale, "say stale\n").unwrap();
                temp
            },
            |temp| {
                let stats = run_cmd::sync_dir(
                    &temp.path().join("dist/bench"),
                    &temp.path().join("server/world/datapacks/bench"),
                )
                .unwrap();
                criterion::black_box(stats);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_validation,
    bench_generated_json_parsing,
    bench_component_writing,
    bench_resource_record_writing,
    bench_zip_packaging,
    bench_server_sync
);
criterion_main!(benches);
