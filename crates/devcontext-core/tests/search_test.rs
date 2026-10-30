use devcontext_core::domain::{
    Decision, DecisionStatus, LogEntry, Milestone, MilestoneStatus, Note, Project, RecordType,
    SavedCommand,
};
use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::storage::connection::DbConnection;
use devcontext_core::storage::migrations::m0001_initial::Migration0001Initial;
use devcontext_core::storage::migrations::{Migration, MigrationRunner};
use devcontext_core::storage::SqliteStorage;
use tempfile::tempdir;

#[test]
fn test_search_fts5_migration_and_compatibility() {
    let mut conn = DbConnection::open_in_memory().unwrap();

    // 1. Initialize migration table and run migration v1 only
    MigrationRunner::init_migration_table(&conn).unwrap();
    let v1 = Migration0001Initial;
    let tx = conn.transaction().unwrap();
    v1.up(&tx).unwrap();
    tx.execute(
        "INSERT INTO schema_migrations (version, applied_at, name) VALUES (1, 'now', '0001_initial_schema');",
        [],
    ).unwrap();
    tx.commit().unwrap();

    // Insert data while schema is at version 1
    conn.execute(
        "INSERT INTO projects (id, name, created_at) VALUES ('11111111-1111-1111-1111-111111111111', 'Legacy App', 'now');",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO records (id, project_id, record_type, title, created_at, updated_at, tags, is_deleted)
         VALUES ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111', 'note', 'Legacy Migration Note', 'now', 'now', '[\"legacy\"]', 0);",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO notes (record_id, content_markdown) VALUES ('22222222-2222-2222-2222-222222222222', 'Pre-existing legacy content before FTS5 migration.');",
        [],
    ).unwrap();

    // 2. Run pending migrations to upgrade to v2
    let applied = MigrationRunner::run_pending(&mut conn).unwrap();
    assert_eq!(applied, 1);
    assert_eq!(MigrationRunner::current_version(&conn).unwrap(), 2);

    // 3. Verify existing legacy record is immediately indexed and searchable
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records_fts WHERE records_fts MATCH 'legacy';",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_search_sync_insert_update_soft_delete_and_restore() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Sync Test Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // 1. Insert note
    let mut note = Note::new(
        project.id,
        "Original Note Title",
        "Initial markdown content discussing SQLite indexing.",
        vec!["database".into()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    let filter = SearchFilter::default();

    // Search matches inserted note
    let matches = SearchService::search(&storage, project.id, root, "indexing", &filter).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].title, "Original Note Title");

    // 2. Update note body
    note.content_markdown = "Updated markdown content discussing Redis caching.".into();
    storage.save_note(&note).unwrap();

    // Old term no longer matches
    let old_matches =
        SearchService::search(&storage, project.id, root, "indexing", &filter).unwrap();
    assert_eq!(old_matches.len(), 0);

    // New term matches
    let new_matches =
        SearchService::search(&storage, project.id, root, "caching", &filter).unwrap();
    assert_eq!(new_matches.len(), 1);

    // 3. Update title
    note.record.title = "Redis Strategy Note".into();
    storage.save_note(&note).unwrap();
    let title_matches =
        SearchService::search(&storage, project.id, root, "Strategy", &filter).unwrap();
    assert_eq!(title_matches.len(), 1);
    assert_eq!(title_matches[0].title, "Redis Strategy Note");

    // 4. Soft delete record
    let deleted = storage.soft_delete_record(note.record.id).unwrap();
    assert!(deleted);

    // Must not appear in search results when soft deleted
    let deleted_matches =
        SearchService::search(&storage, project.id, root, "Redis", &filter).unwrap();
    assert_eq!(deleted_matches.len(), 0);

    // 5. Restore record (is_deleted = false)
    note.record.is_deleted = false;
    storage.save_note(&note).unwrap();

    let restored_matches =
        SearchService::search(&storage, project.id, root, "Redis", &filter).unwrap();
    assert_eq!(restored_matches.len(), 1);
    assert_eq!(restored_matches[0].title, "Redis Strategy Note");

    // 6. Hard delete record
    let hard_deleted = storage.hard_delete_record(note.record.id).unwrap();
    assert!(hard_deleted);

    let hard_matches = SearchService::search(&storage, project.id, root, "Redis", &filter).unwrap();
    assert_eq!(hard_matches.len(), 0);
}

#[test]
fn test_search_across_all_record_types_and_type_filtering() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("All Types Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // 1. Note
    let note = Note::new(
        project.id,
        "Observability Guide",
        "Configuring telemetry and metrics for microservices.",
        vec!["monitoring".into()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    // 2. Decision
    let decision = Decision::new(
        project.id,
        "ADR 0012: Telemetry Pipeline",
        DecisionStatus::Accepted,
        "Telemetry data volume is growing rapidly.",
        "Adopt Prometheus for telemetry collection.",
        "Requires dedicated storage node.",
        None,
        vec!["monitoring".into()],
    )
    .unwrap();
    storage.save_decision(&decision).unwrap();

    // 3. Milestone
    let milestone = Milestone::new(
        project.id,
        "Telemetry Rollout",
        MilestoneStatus::Active,
        None,
        Some("Deliver monitoring and telemetry infrastructure to production.".into()),
        vec!["ops".into()],
    )
    .unwrap();
    storage.save_milestone(&milestone).unwrap();

    // 4. Log Entry
    let log = LogEntry::new(
        project.id,
        "Telemetry Ingestion Deployed",
        "Successfully deployed telemetry collector agents.",
        Some("Collector logs indicate zero dropped packets.".into()),
        None,
        vec![],
    )
    .unwrap();
    storage.save_log_entry(&log).unwrap();

    // 5. Saved Command
    let cmd = SavedCommand::new(
        project.id,
        "Check Telemetry Agent Status",
        "systemctl status telemetry-agent",
        Some("Inspect running telemetry service".into()),
        None,
        vec![],
    )
    .unwrap();
    storage.save_command(&cmd).unwrap();

    // Search across ALL types for common term "telemetry"
    let all_filter = SearchFilter::default();
    let all_results =
        SearchService::search(&storage, project.id, root, "telemetry", &all_filter).unwrap();
    assert_eq!(all_results.len(), 5);

    // Verify record types present
    assert!(all_results
        .iter()
        .any(|r| r.record_type == RecordType::Note));
    assert!(all_results
        .iter()
        .any(|r| r.record_type == RecordType::Decision));
    assert!(all_results
        .iter()
        .any(|r| r.record_type == RecordType::Milestone));
    assert!(all_results
        .iter()
        .any(|r| r.record_type == RecordType::LogEntry));
    assert!(all_results
        .iter()
        .any(|r| r.record_type == RecordType::SavedCommand));

    // Filter by type: Decision only
    let decision_filter = SearchFilter {
        record_type: Some(RecordType::Decision),
        limit: None,
    };
    let decision_results =
        SearchService::search(&storage, project.id, root, "telemetry", &decision_filter).unwrap();
    assert_eq!(decision_results.len(), 1);
    assert_eq!(decision_results[0].record_type, RecordType::Decision);
    assert_eq!(decision_results[0].title, "ADR 0012: Telemetry Pipeline");

    // Filter by type: Command only
    let cmd_filter = SearchFilter {
        record_type: Some(RecordType::SavedCommand),
        limit: None,
    };
    let cmd_results =
        SearchService::search(&storage, project.id, root, "telemetry", &cmd_filter).unwrap();
    assert_eq!(cmd_results.len(), 1);
    assert_eq!(cmd_results[0].record_type, RecordType::SavedCommand);
    assert_eq!(cmd_results[0].title, "Check Telemetry Agent Status");
}

#[test]
fn test_search_unicode_and_turkish_characters() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Turkish Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(
        project.id,
        "Türkçe Güvenlik Dokümanı",
        "Kullanıcı şifreleme ve kimlik doğrulama süreçleri açıklandı. Öğrenci işleri veritabanı.",
        vec!["güvenlik".into()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    let filter = SearchFilter::default();

    // Exact Turkish characters search
    let r1 = SearchService::search(&storage, project.id, root, "şifreleme", &filter).unwrap();
    assert_eq!(r1.len(), 1);

    let r2 = SearchService::search(&storage, project.id, root, "öğrenci", &filter).unwrap();
    assert_eq!(r2.len(), 1);

    // Diacritic-insensitive matching: "guvenlik" matches "güvenlik"
    let r3 = SearchService::search(&storage, project.id, root, "guvenlik", &filter).unwrap();
    assert_eq!(r3.len(), 1);

    // "sifreleme" matches "şifreleme"
    let r4 = SearchService::search(&storage, project.id, root, "sifreleme", &filter).unwrap();
    assert_eq!(r4.len(), 1);

    // "ogrenci" matches "öğrenci"
    let r5 = SearchService::search(&storage, project.id, root, "ogrenci", &filter).unwrap();
    assert_eq!(r5.len(), 1);
}

#[test]
fn test_turkish_character_casing_and_normalization() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Turkish Casing Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(
        project.id,
        "Türkçe Karakter ve Büyük-Küçük Harf Testi",
        "Isparta ılık ıspanak. İstanbul istasyon. Şebeke şeker. Ağaç yağmur. Üzüm üretim. Ördek örnek. Çınar çekiç.",
        vec!["türkçe".into()],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    let filter = SearchFilter::default();

    // S / s (Ş / ş)
    assert_eq!(
        SearchService::search(&storage, project.id, root, "şebeke", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Şebeke", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ŞEBEKE", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "sebeke", &filter)
            .unwrap()
            .len(),
        1
    );

    // C / c (Ç / ç)
    assert_eq!(
        SearchService::search(&storage, project.id, root, "çekiç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Çekiç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÇEKİÇ", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "cekic", &filter)
            .unwrap()
            .len(),
        1
    );

    // G / g (Ğ / ğ)
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ağaç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Ağaç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "AĞAÇ", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "agac", &filter)
            .unwrap()
            .len(),
        1
    );

    // U / u (Ü / ü)
    assert_eq!(
        SearchService::search(&storage, project.id, root, "üretim", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Üretim", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÜRETİM", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "uretim", &filter)
            .unwrap()
            .len(),
        1
    );

    // O / o (Ö / ö)
    assert_eq!(
        SearchService::search(&storage, project.id, root, "örnek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Örnek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÖRNEK", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ornek", &filter)
            .unwrap()
            .len(),
        1
    );

    // I / ı and İ / i
    // Verify original stored content is preserved without alterations
    let found = SearchService::search(&storage, project.id, root, "isparta", &filter).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].title, "Türkçe Karakter ve Büyük-Küçük Harf Testi");

    // Comprehensive assertions for all 7 Turkish character pairs across upper/lower/diacritic variants:

    // 1. Ş / ş
    assert_eq!(
        SearchService::search(&storage, project.id, root, "şebeke", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Şebeke", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ŞEBEKE", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "sebeke", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "SEBEKE", &filter)
            .unwrap()
            .len(),
        1
    );

    // 2. Ç / ç
    assert_eq!(
        SearchService::search(&storage, project.id, root, "çekiç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Çekiç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÇEKİÇ", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "cekic", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "CEKIC", &filter)
            .unwrap()
            .len(),
        1
    );

    // 3. Ğ / ğ
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ağaç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Ağaç", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "AĞAÇ", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "agac", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "AGAC", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "yağmur", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "YAĞMUR", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "yagmur", &filter)
            .unwrap()
            .len(),
        1
    );

    // 4. Ü / ü
    assert_eq!(
        SearchService::search(&storage, project.id, root, "üretim", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Üretim", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÜRETİM", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "uretim", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "URETIM", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "üzüm", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Üzüm", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "uzum", &filter)
            .unwrap()
            .len(),
        1
    );

    // 5. Ö / ö
    assert_eq!(
        SearchService::search(&storage, project.id, root, "örnek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Örnek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ÖRNEK", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ornek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ORNEK", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ördek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Ördek", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ordek", &filter)
            .unwrap()
            .len(),
        1
    );

    // 6. I / ı
    assert_eq!(
        SearchService::search(&storage, project.id, root, "isparta", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ısparta", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Isparta", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ISPARTA", &filter)
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        SearchService::search(&storage, project.id, root, "ıspanak", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Ispanak", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ISPANAK", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ispanak", &filter)
            .unwrap()
            .len(),
        1
    );

    assert_eq!(
        SearchService::search(&storage, project.id, root, "ılık", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ILIK", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "Ilık", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ilik", &filter)
            .unwrap()
            .len(),
        1
    );

    // 7. İ / i
    assert_eq!(
        SearchService::search(&storage, project.id, root, "istanbul", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "İstanbul", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "İSTANBUL", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ISTANBUL", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "istasyon", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "İstasyon", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "İSTASYON", &filter)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "ISTASYON", &filter)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn test_search_invalid_syntax_and_empty_queries() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Resilience Project", None).unwrap();
    storage.save_project(&project).unwrap();

    let note = Note::new(
        project.id,
        "Authentication Architecture",
        "OAuth2 and JWT token verification system.",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note).unwrap();

    let filter = SearchFilter::default();

    // 1. Empty query returns empty Vec without hitting database
    assert_eq!(
        SearchService::search(&storage, project.id, root, "", &filter)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        SearchService::search(&storage, project.id, root, "   ", &filter)
            .unwrap()
            .len(),
        0
    );

    // 2. Unclosed quotes must not crash SQLite or leak syntax error
    let unclosed =
        SearchService::search(&storage, project.id, root, "\"Authentication", &filter).unwrap();
    assert_eq!(unclosed.len(), 1);

    // 3. FTS5 bare operators must not panic or leak raw syntax error
    let op_query =
        SearchService::search(&storage, project.id, root, "AND NOT OR", &filter).unwrap();
    assert_eq!(op_query.len(), 0);

    // 4. Special punctuation only
    let punct = SearchService::search(&storage, project.id, root, "*** ::: ???", &filter).unwrap();
    assert_eq!(punct.len(), 0);
}

#[test]
fn test_search_project_isolation() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let p1 = Project::new("Project Alpha", None).unwrap();
    storage.save_project(&p1).unwrap();

    let p2 = Project::new("Project Beta", None).unwrap();
    storage.save_project(&p2).unwrap();

    let note_p1 = Note::new(
        p1.id,
        "Confidential Alpha Plan",
        "Secret formula for Project Alpha.",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note_p1).unwrap();

    let note_p2 = Note::new(
        p2.id,
        "Confidential Beta Plan",
        "Secret formula for Project Beta.",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note_p2).unwrap();

    let filter = SearchFilter::default();

    // Query in Project 1 returns only Project 1 records
    let r1 = SearchService::search(&storage, p1.id, root, "Secret", &filter).unwrap();
    assert_eq!(r1.len(), 1);
    assert_eq!(r1[0].title, "Confidential Alpha Plan");

    // Query in Project 2 returns only Project 2 records
    let r2 = SearchService::search(&storage, p2.id, root, "Secret", &filter).unwrap();
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0].title, "Confidential Beta Plan");
}

#[test]
fn test_search_limits_and_bm25_title_weighting() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Ranking Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // Item A has match in body only
    let note_a = Note::new(
        project.id,
        "General Discussion",
        "Discussing Kubernetes cluster orchestration.",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note_a).unwrap();

    // Item B has match in Title (weighted 5x higher)
    let note_b = Note::new(
        project.id,
        "Kubernetes Best Practices",
        "Container guidelines and deployment templates.",
        vec![],
        None,
    )
    .unwrap();
    storage.save_note(&note_b).unwrap();

    // Search for "Kubernetes"
    let filter = SearchFilter::default();
    let results = SearchService::search(&storage, project.id, root, "Kubernetes", &filter).unwrap();
    assert_eq!(results.len(), 2);

    // Title match must rank first (lower score = higher relevance)
    assert_eq!(results[0].title, "Kubernetes Best Practices");
    assert_eq!(results[1].title, "General Discussion");
    assert!(results[0].score <= results[1].score);

    // Limit filter
    let limit_filter = SearchFilter {
        limit: Some(1),
        ..Default::default()
    };
    let limited =
        SearchService::search(&storage, project.id, root, "Kubernetes", &limit_filter).unwrap();
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].title, "Kubernetes Best Practices");
}

#[test]
fn test_search_transaction_rollback_consistency() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let project = Project::new("Rollback Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // Intentionally roll back a note insertion inside a transaction
    let res: devcontext_core::error::Result<()> = storage.with_transaction(|tx| {
        tx.execute(
            "INSERT INTO records (id, project_id, record_type, title, created_at, updated_at, tags, is_deleted)
             VALUES ('33333333-3333-3333-3333-333333333333', ?1, 'note', 'Aborted Note', 'now', 'now', '[]', 0);",
            rusqlite::params![project.id.to_string()],
        )?;
        tx.execute(
            "INSERT INTO notes (record_id, content_markdown) VALUES ('33333333-3333-3333-3333-333333333333', 'Phantom content to be rolled back');",
            [],
        )?;
        // Trigger simulated rollback
        Err(devcontext_core::error::CoreError::Validation("Simulated abort".into()))
    });
    assert!(res.is_err());

    // Verify FTS5 index does not contain the rolled-back note
    let filter = SearchFilter::default();
    let matches = SearchService::search(
        &storage,
        project.id,
        std::path::Path::new("."),
        "Phantom",
        &filter,
    )
    .unwrap();
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_search_reindex_and_performance_with_realistic_fixture() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let storage = SqliteStorage::open_in_memory().unwrap();

    let project = Project::new("Performance Project", None).unwrap();
    storage.save_project(&project).unwrap();

    // 1. Insert 100 records
    for i in 1..=100 {
        let note = Note::new(
            project.id,
            format!("Performance Note #{}", i),
            format!(
                "This is the markdown body of note number {} containing architectural details and keywords.",
                i
            ),
            vec!["benchmark".into(), format!("tag-{}", i % 10)],
            None,
        )
        .unwrap();
        storage.save_note(&note).unwrap();
    }

    // 2. Test reindex
    // Wipe virtual table manually
    storage
        .with_transaction(|tx| {
            tx.execute("DELETE FROM records_fts;", [])?;
            Ok(())
        })
        .unwrap();

    let filter = SearchFilter::default();
    assert_eq!(
        SearchService::search(&storage, project.id, root, "architectural", &filter)
            .unwrap()
            .len(),
        0
    );

    // Rebuild index
    let count = SearchService::rebuild_index(&storage).unwrap();
    assert_eq!(count, 100);

    // 3. Search performance check
    let start = std::time::Instant::now();
    let results =
        SearchService::search(&storage, project.id, root, "architectural", &filter).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 50); // Default limit is 50
    assert!(
        elapsed.as_millis() < 50,
        "FTS5 search should take < 50ms, took {}ms",
        elapsed.as_millis()
    );
}
