//! Phase 8: Performance Measurements and Scaling Benchmarks.

use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::workspace::WorkspaceManager;
use std::time::Instant;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_search_latency_on_10k_records_benchmark() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let (project, storage) =
        WorkspaceManager::initialize_project(root, Some("ScaleBenchmarkProject")).unwrap();

    let record_count = 10_000;
    println!("\n--- Phase 8 Performance Benchmark: 10,000 Records ---");

    // 1. Batch populate 10,000 realistic records inside a single transaction
    // Note: SQLite trigger `trg_fts_notes_insert` automatically maintains the FTS5 index.
    let t_insert_start = Instant::now();
    storage
        .with_transaction(|tx| {
            let mut stmt_rec = tx.prepare(
                "INSERT INTO records (id, project_id, record_type, title, is_deleted, created_at, updated_at, tags)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7)",
            )?;
            let mut stmt_note = tx.prepare(
                "INSERT INTO notes (record_id, content_markdown, category, is_pinned, is_archived)
                 VALUES (?1, ?2, ?3, 0, 0)",
            )?;

            for i in 1..=record_count {
                let id = Uuid::new_v4().to_string();
                let title = format!("Architecture Decision #{}: Subsystem Optimization Layer", i);
                let body = format!(
                    "Technical details for subsystem unit {}: utilizing SQLite WAL concurrency, FTS5 BM25 search indices, and low-latency cache buffers for maximum responsiveness.",
                    i
                );
                let tags = format!("[\"benchmark\", \"tag-{}\", \"optimization\"]", i % 20);
                let created_at = "2026-10-31T00:00:00Z";

                stmt_rec.execute((
                    &id,
                    &project.id.to_string(),
                    "note",
                    &title,
                    created_at,
                    created_at,
                    &tags,
                ))?;

                stmt_note.execute((&id, &body, "architecture"))?;
            }
            Ok(())
        })
        .unwrap();

    let insert_duration = t_insert_start.elapsed();
    println!(
        "Dataset creation: 10,000 records indexed in {:.2}s ({:.1} records/sec)",
        insert_duration.as_secs_f64(),
        (record_count as f64) / insert_duration.as_secs_f64()
    );

    // 2. Warm up search query
    let filter = SearchFilter::default();
    let warmup_res =
        SearchService::search(&storage, project.id, root, "concurrency", &filter).unwrap();
    assert!(
        !warmup_res.is_empty(),
        "Warmup search should return matches"
    );

    // 3. Measure search latency across 10 distinct queries
    let test_queries = [
        "concurrency",
        "optimization",
        "responsiveness",
        "architecture",
        "subsystem",
        "buffers",
        "sqlite",
        "latency",
        "indices",
        "maximum",
    ];

    let mut latencies_ms = Vec::new();

    for query in &test_queries {
        let t0 = Instant::now();
        let results = SearchService::search(&storage, project.id, root, query, &filter).unwrap();
        let elapsed = t0.elapsed();
        let ms = elapsed.as_secs_f64() * 1000.0;
        latencies_ms.push(ms);

        assert!(
            !results.is_empty(),
            "Query '{}' must return results in 10k dataset",
            query
        );
        assert!(
            results.len() <= 50,
            "Query '{}' results must be bounded by limit",
            query
        );
    }

    let min_ms = latencies_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max_ms = latencies_ms.iter().copied().fold(0.0, f64::max);
    let avg_ms = latencies_ms.iter().sum::<f64>() / (latencies_ms.len() as f64);

    println!(
        "Search Latency (10k records, 10 queries): min={:.2}ms, avg={:.2}ms, max={:.2}ms",
        min_ms, avg_ms, max_ms
    );

    // ROADMAP target: under 300 ms for 10,000 records
    assert!(
        avg_ms < 300.0,
        "Target violated: Search latency on 10,000 records must be < 300ms, observed avg: {:.2}ms",
        avg_ms
    );
    assert!(
        max_ms < 300.0,
        "Target violated: Worst-case search latency must be < 300ms, observed max: {:.2}ms",
        max_ms
    );
}
