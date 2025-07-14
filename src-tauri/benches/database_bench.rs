use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tollo_lib::database::populate_channels;
use tollo_lib::m3u_parser::Channel;
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    
    conn.execute(
        "CREATE TABLE channels (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE favorites (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE TABLE history (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            logo TEXT NOT NULL,
            url TEXT NOT NULL,
            group_title TEXT NOT NULL,
            tvg_id TEXT NOT NULL,
            resolution TEXT NOT NULL,
            extra_info TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();
    
    conn.execute(
        "CREATE VIRTUAL TABLE channels_fts USING fts5(name, content='channels', content_rowid='id')",
        [],
    ).unwrap();
    
    conn
}

fn create_test_channels(count: usize) -> Vec<Channel> {
    (0..count).map(|i| Channel {
        name: format!("Test Channel {}", i),
        logo: format!("http://example.com/logo{}.png", i),
        url: format!("http://example.com/stream{}", i),
        group_title: format!("Group {}", i % 10),
        tvg_id: format!("test{}", i),
        resolution: "1080p".to_string(),
        extra_info: format!("Extra info {}", i),
    }).collect()
}

fn create_realistic_channels(count: usize) -> Vec<Channel> {
    let channel_names = [
        "CNN International", "BBC World News", "ESPN Sports Center", "Discovery Channel",
        "National Geographic", "MTV Music", "Fox News", "Sky Sports", "HBO Movies",
        "Cartoon Network", "Disney Channel", "Animal Planet", "History Channel",
        "Comedy Central", "Nickelodeon", "Food Network", "Travel Channel"
    ];
    
    let groups = [
        "News", "Sports", "Entertainment", "Movies", "Documentary", "Kids", "Music"
    ];
    
    (0..count).map(|i| {
        let base_name = channel_names[i % channel_names.len()];
        let group = groups[i % groups.len()];
        let name = if i >= channel_names.len() {
            format!("{} {}", base_name, (i / channel_names.len()) + 1)
        } else {
            base_name.to_string()
        };
        
        Channel {
            name,
            logo: format!("http://example.com/logo{}.png", i),
            url: format!("http://example.com/stream{}.m3u8", i),
            group_title: group.to_string(),
            tvg_id: format!("ch{}", i),
            resolution: if i % 3 == 0 { "1080p".to_string() } else { "720p".to_string() },
            extra_info: if i % 5 == 0 { format!("[HD] Extra info {}", i) } else { "".to_string() },
        }
    }).collect()
}

fn bench_populate_channels(c: &mut Criterion) {
    let mut group = c.benchmark_group("populate_channels");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("simple_channels", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut conn = create_test_db();
                    let channels = create_test_channels(size);
                    populate_channels(&mut conn, black_box(&channels)).unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("realistic_channels", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut conn = create_test_db();
                    let channels = create_realistic_channels(size);
                    populate_channels(&mut conn, black_box(&channels)).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn bench_database_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_operations");
    
    // Setup database with test data
    let mut conn = create_test_db();
    let channels = create_realistic_channels(1000);
    populate_channels(&mut conn, &channels).unwrap();
    
    // Benchmark basic database operations
    group.bench_function("simple_query", |b| {
        b.iter(|| {
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM channels",
                [],
                |row| row.get(0)
            ).unwrap();
            black_box(count)
        });
    });
    
    group.finish();
}

fn bench_database_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_queries");
    
    // Setup database with test data
    let mut conn = create_test_db();
    let channels = create_realistic_channels(5000);
    populate_channels(&mut conn, &channels).unwrap();
    
    // Benchmark different query patterns
    group.bench_function("filter_by_group", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT * FROM channels WHERE group_title = ?1"
            ).unwrap();
            
            let channel_iter = stmt.query_map(["News"], |row| {
                Ok(Channel {
                    name: row.get(1)?,
                    logo: row.get(2)?,
                    url: row.get(3)?,
                    group_title: row.get(4)?,
                    tvg_id: row.get(5)?,
                    resolution: row.get(6)?,
                    extra_info: row.get(7)?,
                })
            }).unwrap();
            
            let results: Vec<Channel> = channel_iter.collect::<Result<Vec<_>, _>>().unwrap();
            black_box(results)
        });
    });
    
    group.bench_function("search_by_name_pattern", |b| {
        b.iter(|| {
            let mut stmt = conn.prepare(
                "SELECT * FROM channels WHERE name LIKE ?1 LIMIT 50"
            ).unwrap();
            
            let channel_iter = stmt.query_map(["%CNN%"], |row| {
                Ok(Channel {
                    name: row.get(1)?,
                    logo: row.get(2)?,
                    url: row.get(3)?,
                    group_title: row.get(4)?,
                    tvg_id: row.get(5)?,
                    resolution: row.get(6)?,
                    extra_info: row.get(7)?,
                })
            }).unwrap();
            
            let results: Vec<Channel> = channel_iter.collect::<Result<Vec<_>, _>>().unwrap();
            black_box(results)
        });
    });
    
    group.bench_function("count_by_group", |b| {
        b.iter(|| {
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM channels WHERE group_title = ?1",
                ["Sports"],
                |row| row.get(0)
            ).unwrap();
            black_box(count)
        });
    });
    
    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    // Test batch inserts vs individual inserts
    group.bench_function("individual_inserts", |b| {
        b.iter_batched(
            || {
                let conn = create_test_db();
                let channels = create_test_channels(100);
                (conn, channels)
            },
            |(mut conn, channels)| {
                for channel in channels {
                    conn.execute(
                        "INSERT INTO channels (name, logo, url, group_title, tvg_id, resolution, extra_info) 
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![
                            channel.name,
                            channel.logo,
                            channel.url,
                            channel.group_title,
                            channel.tvg_id,
                            channel.resolution,
                            channel.extra_info
                        ],
                    ).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.bench_function("batch_insert", |b| {
        b.iter_batched(
            || {
                let conn = create_test_db();
                let channels = create_test_channels(100);
                (conn, channels)
            },
            |(mut conn, channels)| {
                populate_channels(&mut conn, black_box(&channels)).unwrap()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_populate_channels,
    bench_database_operations,
    bench_database_queries,
    bench_batch_operations
);
criterion_main!(benches);