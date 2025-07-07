# Testing Guide for Töllö

This document provides comprehensive information about the testing infrastructure implemented for the Töllö IPTV player's Rust backend.

## Overview

The Töllö backend includes a comprehensive testing suite with 138+ tests covering all critical functionality, plus performance benchmarks to monitor system performance and detect regressions.

## Test Categories

### 1. Unit Tests
- **Database Operations** (15 tests) - Core database functionality
- **Error Handling** (8 tests) - TolloError enum and error scenarios  
- **M3U Parser** (16 tests + property-based) - Playlist parsing logic
- **Fuzzy Search** (28 tests + property-based) - Search algorithms and scoring
- **Cache Operations** (41 tests) - Search cache and image cache functionality
- **Integration Tests** (17 tests) - Tauri command handlers with mocking
- **Error Scenarios** (25 tests) - Edge cases and malformed input handling

### 2. Property-Based Tests
- Complex logic validation using `proptest`
- Automatically generates test cases to find edge cases
- Covers M3U parsing and fuzzy search algorithms

### 3. Performance Benchmarks
- **Fuzzy Search Benchmarks** - Scaling, query types, algorithms
- **Database Benchmarks** - Population, queries, batch operations  
- **M3U Parser Benchmarks** - Simple/complex parsing, progress callbacks
- **Cache Benchmarks** - Search cache, image cache, memory usage

## Running Tests

### Prerequisites
Ensure you're in the Rust backend directory:
```bash
cd src-tauri
```

### Unit Tests

#### Run All Tests
```bash
cargo test
```

#### Run Specific Test Modules
```bash
# Database tests
cargo test database

# M3U parser tests  
cargo test m3u_parser

# Fuzzy search tests
cargo test fuzzy_search

# Cache operation tests
cargo test search
cargo test image_cache

# Integration tests
cargo test integration_tests

# Error handling tests
cargo test error
```

#### Run Tests with Output
```bash
# Show test output (useful for debugging)
cargo test -- --nocapture

# Run tests in sequence (not parallel)
cargo test -- --test-threads=1
```

### Performance Benchmarks

#### Run All Benchmarks
```bash
cargo bench
```

#### Run Specific Benchmarks
```bash
# Fuzzy search performance
cargo bench --bench fuzzy_search_bench

# Database performance
cargo bench --bench database_bench

# M3U parser performance
cargo bench --bench m3u_parser_bench

# Cache performance
cargo bench --bench cache_bench
```

#### Quick Benchmark Run
```bash
# Run benchmarks quickly (fewer iterations)
cargo bench -- --quick
```

## Test Details

### Database Tests (`src/database.rs`)
- **populate_channels** - Channel insertion and updating
- **SQL injection prevention** - Parameterized queries safety
- **Unicode handling** - International character support
- **Constraint validation** - Database integrity checks
- **Error scenarios** - Malformed data and edge cases

### M3U Parser Tests (`src/m3u_parser.rs`)
- **Basic parsing** - Standard M3U format support
- **Attribute extraction** - tvg-id, logo, group-title parsing
- **Progress callbacks** - Large file processing with progress
- **VLC options** - Extended M3U format support
- **Edge cases** - Malformed files, missing attributes
- **Property-based testing** - Automated edge case discovery

### Fuzzy Search Tests (`src/fuzzy_search.rs`)
- **Algorithm correctness** - Scoring and ranking accuracy
- **Performance scaling** - Large dataset handling
- **Query types** - Single word, multi-word, fuzzy matching
- **Case sensitivity** - Configurable matching behavior
- **Result limiting** - Pagination and performance optimization
- **Multi-language support** - Unicode and international content

### Cache Operation Tests (`src/search.rs`, `src/image_cache.rs`)
- **LRU eviction** - Cache size management
- **TTL expiration** - Time-based cache invalidation
- **Prefix matching** - Smart cache utilization
- **Hash consistency** - Deterministic cache keys
- **Concurrent access** - Thread safety validation
- **Memory management** - Cache cleanup and optimization

### Integration Tests (`src/integration_tests.rs`)
- **Tauri command handlers** - Full command testing with mocking
- **State management** - Shared state across commands
- **Error propagation** - Proper error handling in commands
- **Database integration** - Real database operations
- **Cache integration** - Cache state management

## Performance Baselines

### Current Performance Metrics
- **M3U Parsing**: ~6ms for 5,000 channels
- **Fuzzy Search**: ~6ms for 10,000 channels
- **Database Population**: ~40ms for 10,000 channels
- **Cache Operations**: <1ms for most operations

### Benchmark Categories
- **Scalability**: Performance with increasing data sizes
- **Algorithm efficiency**: Different implementation approaches
- **Memory usage**: Cache and data structure optimization
- **Concurrent operations**: Multi-threaded performance

## Continuous Integration

### Running Tests in CI
```bash
# Run all tests with coverage
cargo test --all-features

# Run benchmarks (comparison mode)
cargo bench --bench fuzzy_search_bench -- --save-baseline main

# Compare benchmark results
cargo bench --bench fuzzy_search_bench -- --load-baseline main
```

### Test Coverage
To generate test coverage reports:
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html
```

## Writing New Tests

### Test Structure
Tests are organized using Rust's built-in testing framework with additional libraries:
- `#[cfg(test)]` modules for unit tests
- `criterion` for performance benchmarks
- `proptest` for property-based testing
- `tokio-test` for async testing
- `tempfile` for temporary file management

### Example Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_function() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = example_function(input);
        
        // Assert
        assert_eq!(result, "expected output");
    }
}
```

### Example Benchmark
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_example_function(c: &mut Criterion) {
    c.bench_function("example_function", |b| {
        b.iter(|| {
            example_function(black_box("test input"))
        });
    });
}

criterion_group!(benches, bench_example_function);
criterion_main!(benches);
```

## Troubleshooting

### Common Issues

#### Test Failures
```bash
# Run a specific failing test with detailed output
cargo test test_name -- --nocapture --exact
```

#### Benchmark Issues
```bash
# Clean benchmark cache
rm -rf target/criterion

# Run benchmarks with more iterations
cargo bench -- --warm-up-time 3 --measurement-time 10
```

#### Dependencies
```bash
# Update test dependencies
cargo update

# Clean and rebuild
cargo clean && cargo build
```

### Performance Regression Detection
- Monitor benchmark results over time
- Set up alerts for performance degradation >10%
- Use `--save-baseline` and `--load-baseline` for comparisons

## Best Practices

1. **Test Isolation**: Each test should be independent and not rely on external state
2. **Mock External Dependencies**: Use mocks for database connections, file systems, network calls
3. **Property-Based Testing**: Use `proptest` for complex algorithms to discover edge cases
4. **Performance Monitoring**: Regular benchmark runs to detect regressions
5. **Error Path Testing**: Test both success and failure scenarios
6. **Documentation**: Keep tests readable and well-documented

## Test Statistics

- **Total Tests**: 138+ across all modules
- **Benchmark Suites**: 4 comprehensive suites
- **Coverage Areas**: Database, parsing, search, cache, integration, error handling
- **Performance Baselines**: Established for all critical paths
- **Property-Based Tests**: Automated edge case discovery
- **Error Scenarios**: Comprehensive edge case coverage

This testing infrastructure ensures code quality, performance monitoring, and regression detection for the Töllö backend.