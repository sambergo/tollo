use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tollo_lib::fuzzy_search::*;
use tollo_lib::m3u_parser::Channel;

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
        "Comedy Central", "Nickelodeon", "Food Network", "Travel Channel",
        "Science Channel", "Discovery Science", "Eurosport", "CNN Türk",
        "TRT Haber", "Show TV", "Kanal D", "ATV", "Star TV", "TV8", "FOX Türkiye",
        "NTV", "Habertürk", "Bloomberg HT", "CNBC-e", "TRT Spor", "Bein Sports",
        "Smart Spor", "NBA TV", "Setanta Sports", "Eurosport 2", "Motor TV"
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
            url: format!("http://example.com/stream{}", i),
            group_title: group.to_string(),
            tvg_id: format!("ch{}", i),
            resolution: if i % 3 == 0 { "1080p".to_string() } else { "720p".to_string() },
            extra_info: "".to_string(),
        }
    }).collect()
}

fn bench_fuzzy_search_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search_scaling");
    
    // Test scaling with different dataset sizes
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let channels = create_realistic_channels(*size);
        
        group.bench_with_input(
            BenchmarkId::new("search_cnn", size),
            &channels,
            |b, channels| {
                b.iter(|| {
                    let matcher = FuzzyMatcher::new();
                    matcher.search_channels(black_box(channels), black_box("CNN"))
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("search_sports", size),
            &channels,
            |b, channels| {
                b.iter(|| {
                    let matcher = FuzzyMatcher::new();
                    matcher.search_channels(black_box(channels), black_box("sports"))
                });
            },
        );
    }
    
    group.finish();
}

fn bench_fuzzy_search_query_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search_query_types");
    
    let channels = create_realistic_channels(5000);
    let matcher = FuzzyMatcher::new();
    
    // Single word queries
    group.bench_function("single_word_exact", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("ESPN"))
        });
    });
    
    group.bench_function("single_word_fuzzy", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("esn"))
        });
    });
    
    // Multi-word queries
    group.bench_function("multi_word_exact", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("BBC World"))
        });
    });
    
    group.bench_function("multi_word_fuzzy", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("bc wrld"))
        });
    });
    
    // Long queries
    group.bench_function("long_query", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("Discovery Channel Science Documentary"))
        });
    });
    
    // Empty and edge case queries
    group.bench_function("empty_query", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box(""))
        });
    });
    
    group.bench_function("single_char", |b| {
        b.iter(|| {
            matcher.search_channels(black_box(&channels), black_box("C"))
        });
    });
    
    group.finish();
}

fn bench_fuzzy_search_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search_algorithms");
    
    let channels = create_realistic_channels(1000);
    
    // Compare case sensitive vs insensitive
    group.bench_function("case_insensitive", |b| {
        b.iter(|| {
            let matcher = FuzzyMatcher::new(); // Default is case insensitive
            matcher.search_channels(black_box(&channels), black_box("cnn"))
        });
    });
    
    // Benchmark scoring algorithm performance
    group.bench_function("scoring_algorithm", |b| {
        b.iter(|| {
            let matcher = FuzzyMatcher::new();
            // This will exercise the scoring algorithm heavily
            matcher.search_channels(black_box(&channels), black_box("news"))
        });
    });
    
    group.finish();
}

fn bench_search_result_limiting(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_result_limiting");
    
    let channels = create_realistic_channels(10000);
    let matcher = FuzzyMatcher::new();
    
    // Test with no limit (return all matches)
    group.bench_function("no_limit", |b| {
        b.iter(|| {
            // Search for a common term that will match many channels
            matcher.search_channels(black_box(&channels), black_box("TV"))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_fuzzy_search_scaling,
    bench_fuzzy_search_query_types,
    bench_fuzzy_search_algorithms,
    bench_search_result_limiting
);
criterion_main!(benches);