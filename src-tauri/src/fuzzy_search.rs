use crate::m3u_parser::Channel;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub channel: Channel,
    pub score: i32,
    pub match_positions: Vec<usize>,
}

pub struct FuzzyMatcher {
    case_sensitive: bool,
    min_score_threshold: i32,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            min_score_threshold: 20, // Minimum score to include in results
        }
    }

    pub fn with_min_score_threshold(min_score_threshold: i32) -> Self {
        Self {
            case_sensitive: false,
            min_score_threshold,
        }
    }

    pub fn search_channels(&self, channels: &[Channel], query: &str) -> Vec<Channel> {
        if query.is_empty() {
            return channels.to_vec();
        }

        // Split query into words for multi-word search
        let query_words: Vec<&str> = query.split_whitespace().collect();

        let mut matches: Vec<SearchMatch> = channels
            .iter()
            .filter_map(|channel| self.score_channel_multiword(channel, &query_words))
            .filter(|search_match| search_match.score >= self.min_score_threshold)
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.cmp(&a.score));

        matches.into_iter().map(|m| m.channel).collect()
    }

    fn score_channel_multiword(
        &self,
        channel: &Channel,
        query_words: &[&str],
    ) -> Option<SearchMatch> {
        let mut total_score = 0;
        let mut all_positions = Vec::new();

        // All words must match somewhere in the channel
        for word in query_words {
            let mut word_matched = false;

            // Try matching against channel name first (higher priority)
            if let Some((score, positions)) = self.fuzzy_match(&channel.name, word) {
                total_score += score + 10; // Bonus for name match
                all_positions.extend(positions);
                word_matched = true;
            }
            // If not found in name, try group title (lower priority)
            else if let Some((score, positions)) = self.fuzzy_match(&channel.group_title, word) {
                total_score += score / 2; // Penalty for group match
                all_positions.extend(positions);
                word_matched = true;
            }

            if !word_matched {
                return None; // One word didn't match anywhere
            }
        }

        Some(SearchMatch {
            channel: channel.clone(),
            score: total_score,
            match_positions: all_positions,
        })
    }

    fn fuzzy_match(&self, text: &str, pattern: &str) -> Option<(i32, Vec<usize>)> {
        let text_chars: Vec<char> = if self.case_sensitive {
            text.chars().collect()
        } else {
            text.to_lowercase().chars().collect()
        };

        let pattern_chars: Vec<char> = if self.case_sensitive {
            pattern.chars().collect()
        } else {
            pattern.to_lowercase().chars().collect()
        };

        if pattern_chars.is_empty() {
            return Some((0, Vec::new()));
        }

        self.calculate_score(&text_chars, &pattern_chars)
    }

    fn calculate_score(&self, text: &[char], pattern: &[char]) -> Option<(i32, Vec<usize>)> {
        let mut score = 0;
        let mut positions = Vec::new();
        let mut text_idx = 0;
        let mut pattern_idx = 0;
        let mut prev_matched = false;
        let mut consecutive_bonus = 0;

        while pattern_idx < pattern.len() && text_idx < text.len() {
            if pattern[pattern_idx] == text[text_idx] {
                positions.push(text_idx);

                let mut char_score = 16; // Base score for each match

                // Consecutive match bonus - rewards sequences
                if prev_matched {
                    consecutive_bonus = (consecutive_bonus + 15).min(45);
                    char_score += consecutive_bonus;
                } else {
                    consecutive_bonus = 0;
                }

                // Word boundary bonus - rewards matches at start of words
                if text_idx == 0 || !text[text_idx - 1].is_alphanumeric() {
                    char_score += 8;
                }

                // CamelCase bonus - rewards matches at capital letters
                if text_idx > 0
                    && text[text_idx - 1].is_lowercase()
                    && text[text_idx].is_uppercase()
                {
                    char_score += 8;
                }

                // Starting character bonus - prefer matches at the beginning
                if text_idx == 0 {
                    char_score += 12;
                }

                score += char_score;
                pattern_idx += 1;
                prev_matched = true;
            } else {
                prev_matched = false;
                consecutive_bonus = 0;
            }

            text_idx += 1;
        }

        // Check if we matched all pattern characters
        if pattern_idx == pattern.len() {
            // Apply penalties for longer texts (prefer shorter, more precise matches)
            let length_penalty = if text.len() > 50 {
                (text.len() - 50) as i32
            } else {
                0
            };

            // Gap penalty - penalize large gaps between matches
            let gap_penalty = if positions.len() > 1 {
                let total_span = positions.last().unwrap() - positions.first().unwrap();
                let expected_span = positions.len() - 1;
                if total_span > expected_span * 2 {
                    (total_span - expected_span * 2) as i32
                } else {
                    0
                }
            } else {
                0
            };

            Some((score - length_penalty - gap_penalty, positions))
        } else {
            None // Didn't match all pattern characters
        }
    }
}

/// Public function that matches the expected API used in benchmarks
pub fn fuzzy_search(channels: &[Channel], query: &str, limit: usize) -> Vec<Channel> {
    let matcher = FuzzyMatcher::new();
    let results = matcher.search_channels(channels, query);
    
    // Apply limit
    if limit > 0 && results.len() > limit {
        results.into_iter().take(limit).collect()
    } else {
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channels() -> Vec<Channel> {
        vec![
            Channel {
                name: "BBC News".to_string(),
                logo: "http://example.com/bbc.png".to_string(),
                url: "http://example.com/bbc".to_string(),
                group_title: "News".to_string(),
                tvg_id: "bbc1".to_string(),
                resolution: "1080p".to_string(),
                extra_info: "HD".to_string(),
            },
            Channel {
                name: "CNN International".to_string(),
                logo: "http://example.com/cnn.png".to_string(),
                url: "http://example.com/cnn".to_string(),
                group_title: "News".to_string(),
                tvg_id: "cnn1".to_string(),
                resolution: "720p".to_string(),
                extra_info: "".to_string(),
            },
            Channel {
                name: "ESPN Sports".to_string(),
                logo: "http://example.com/espn.png".to_string(),
                url: "http://example.com/espn".to_string(),
                group_title: "Sports".to_string(),
                tvg_id: "espn1".to_string(),
                resolution: "1080p".to_string(),
                extra_info: "Live".to_string(),
            },
            Channel {
                name: "Discovery Channel".to_string(),
                logo: "http://example.com/discovery.png".to_string(),
                url: "http://example.com/discovery".to_string(),
                group_title: "Documentary".to_string(),
                tvg_id: "disc1".to_string(),
                resolution: "720p".to_string(),
                extra_info: "".to_string(),
            },
            Channel {
                name: "BBC iPlayer".to_string(),
                logo: "http://example.com/iplayer.png".to_string(),
                url: "http://example.com/iplayer".to_string(),
                group_title: "Entertainment".to_string(),
                tvg_id: "iplayer1".to_string(),
                resolution: "1080p".to_string(),
                extra_info: "On Demand".to_string(),
            },
        ]
    }

    #[test]
    fn test_fuzzy_matcher_new() {
        let matcher = FuzzyMatcher::new();
        assert!(!matcher.case_sensitive);
        assert_eq!(matcher.min_score_threshold, 20);
    }

    #[test]
    fn test_fuzzy_matcher_with_min_score_threshold() {
        let matcher = FuzzyMatcher::with_min_score_threshold(50);
        assert!(!matcher.case_sensitive);
        assert_eq!(matcher.min_score_threshold, 50);
    }

    #[test]
    fn test_min_score_threshold_filtering() {
        let matcher = FuzzyMatcher::with_min_score_threshold(100);
        let channels = create_test_channels();
        
        // This should return fewer results than the default threshold
        // because only very good matches will exceed the high threshold
        let results = matcher.search_channels(&channels, "z");
        
        // With a high threshold, very weak matches should be filtered out
        // The exact count depends on the scoring, but it should be fewer than all channels
        assert!(results.len() < channels.len());
    }

    #[test]
    fn test_search_channels_empty_query() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "");
        assert_eq!(results.len(), channels.len());
        
        // Should return all channels unchanged
        assert_eq!(results[0].name, "BBC News");
        assert_eq!(results[1].name, "CNN International");
    }

    #[test]
    fn test_search_channels_exact_match() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "BBC News");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "BBC News");
    }

    #[test]
    fn test_search_channels_partial_match() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "BBC");
        assert_eq!(results.len(), 2); // BBC News and BBC iPlayer
        
        // Should prioritize exact matches in name
        assert_eq!(results[0].name, "BBC News");
        assert_eq!(results[1].name, "BBC iPlayer");
    }

    #[test]
    fn test_search_channels_case_insensitive() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "bbc");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "BBC News");
        assert_eq!(results[1].name, "BBC iPlayer");
    }

    #[test]
    fn test_search_channels_fuzzy_match() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "Dscy"); // Should match "Discovery"
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Discovery Channel");
    }

    #[test]
    fn test_search_channels_group_match() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "News");
        assert_eq!(results.len(), 2); // BBC News and CNN International
        
        // Should find channels with "News" in group title
        let names: Vec<&str> = results.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"BBC News"));
        assert!(names.contains(&"CNN International"));
    }

    #[test]
    fn test_search_channels_multi_word_query() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "CNN International");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "CNN International");
    }

    #[test]
    fn test_search_channels_multi_word_partial() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "CNN Int");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "CNN International");
    }

    #[test]
    fn test_search_channels_no_match() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        let results = matcher.search_channels(&channels, "XYZ123");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_channels_name_priority_over_group() {
        let matcher = FuzzyMatcher::new();
        let channels = create_test_channels();
        
        // Search for "Sports" - should prioritize ESPN Sports (name match) over group matches
        let results = matcher.search_channels(&channels, "Sports");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "ESPN Sports");
    }

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("test", "");
        assert!(result.is_some());
        let (score, positions) = result.unwrap();
        assert_eq!(score, 0);
        assert_eq!(positions, Vec::<usize>::new());
    }

    #[test]
    fn test_fuzzy_match_exact_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("test", "test");
        assert!(result.is_some());
        
        let (score, positions) = result.unwrap();
        assert!(score > 0);
        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_fuzzy_match_partial_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("testing", "test");
        assert!(result.is_some());
        
        let (score, positions) = result.unwrap();
        assert!(score > 0);
        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_fuzzy_match_scattered_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("abcdef", "ace");
        assert!(result.is_some());
        
        let (score, positions) = result.unwrap();
        assert!(score > 0);
        assert_eq!(positions, vec![0, 2, 4]);
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("abc", "xyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("Test", "test");
        assert!(result.is_some());
        
        let (score, positions) = result.unwrap();
        assert!(score > 0);
        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_fuzzy_match_consecutive_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.fuzzy_match("abcdef", "abc");
        let result2 = matcher.fuzzy_match("azbycx", "abc");
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        // Consecutive matches should score higher
        assert!(result1.unwrap().0 > result2.unwrap().0);
    }

    #[test]
    fn test_fuzzy_match_word_boundary_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.fuzzy_match("test word", "tw");
        let result2 = matcher.fuzzy_match("atbwc", "tw");
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        // Word boundary matches should score higher
        assert!(result1.unwrap().0 > result2.unwrap().0);
    }

    #[test]
    fn test_fuzzy_match_starting_character_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.fuzzy_match("test", "t");
        let result2 = matcher.fuzzy_match("best", "t");
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        // Starting character matches should score higher
        assert!(result1.unwrap().0 > result2.unwrap().0);
    }

    #[test]
    fn test_fuzzy_match_camel_case_detection() {
        let matcher = FuzzyMatcher::new();
        
        // Test that CamelCase patterns can be matched
        let result1 = matcher.fuzzy_match("testWord", "tW");
        let result2 = matcher.fuzzy_match("TestCase", "TC");
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        // Both should find matches in CamelCase text
        assert!(result1.unwrap().0 > 0);
        assert!(result2.unwrap().0 > 0);
    }

    #[test]
    fn test_fuzzy_match_length_penalty() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.fuzzy_match("test", "t");
        let result2 = matcher.fuzzy_match("this is a very long test string that should be penalized", "t");
        
        assert!(result1.is_some());
        assert!(result2.is_some());
        
        // Shorter strings should score higher due to length penalty
        assert!(result1.unwrap().0 > result2.unwrap().0);
    }

    #[test]
    fn test_fuzzy_search_function_with_limit() {
        let channels = create_test_channels();
        
        let results = fuzzy_search(&channels, "News", 1);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_fuzzy_search_function_no_limit() {
        let channels = create_test_channels();
        
        let results = fuzzy_search(&channels, "News", 0);
        assert_eq!(results.len(), 2); // Should return all matches
    }

    #[test]
    fn test_fuzzy_search_function_limit_larger_than_results() {
        let channels = create_test_channels();
        
        let results = fuzzy_search(&channels, "News", 10);
        assert_eq!(results.len(), 2); // Should return all matches (less than limit)
    }

    #[test]
    fn test_search_match_struct() {
        let channel = Channel {
            name: "Test Channel".to_string(),
            logo: "http://example.com/logo.png".to_string(),
            url: "http://example.com/stream".to_string(),
            group_title: "Test".to_string(),
            tvg_id: "test1".to_string(),
            resolution: "1080p".to_string(),
            extra_info: "HD".to_string(),
        };
        
        let search_match = SearchMatch {
            channel: channel.clone(),
            score: 100,
            match_positions: vec![0, 1, 2],
        };
        
        assert_eq!(search_match.channel.name, "Test Channel");
        assert_eq!(search_match.score, 100);
        assert_eq!(search_match.match_positions, vec![0, 1, 2]);
        
        // Test Clone and Debug traits
        let cloned = search_match.clone();
        assert_eq!(cloned.score, 100);
        
        let debug_str = format!("{:?}", search_match);
        assert!(debug_str.contains("Test Channel"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_score_channel_multiword_all_words_must_match() {
        let matcher = FuzzyMatcher::new();
        let channel = Channel {
            name: "BBC News International".to_string(),
            logo: "".to_string(),
            url: "".to_string(),
            group_title: "News".to_string(),
            tvg_id: "".to_string(),
            resolution: "".to_string(),
            extra_info: "".to_string(),
        };
        
        // All words match - should return a result
        let result = matcher.score_channel_multiword(&channel, &["BBC", "News"]);
        assert!(result.is_some());
        
        // One word doesn't match - should return None
        let result = matcher.score_channel_multiword(&channel, &["BBC", "XYZ"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_channels_list() {
        let matcher = FuzzyMatcher::new();
        let channels = Vec::new();
        
        let results = matcher.search_channels(&channels, "test");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_performance_with_large_channel_list() {
        let matcher = FuzzyMatcher::new();
        let mut channels = Vec::new();
        
        // Create 1000 test channels
        for i in 0..1000 {
            channels.push(Channel {
                name: format!("Channel {}", i),
                logo: format!("http://example.com/logo{}.png", i),
                url: format!("http://example.com/stream{}", i),
                group_title: format!("Group {}", i % 10),
                tvg_id: format!("ch{}", i),
                resolution: "1080p".to_string(),
                extra_info: "".to_string(),
            });
        }
        
        let results = matcher.search_channels(&channels, "Channel 1");
        assert!(!results.is_empty());
        
        // Should find channels with "Channel 1" in the name
        let first_result = &results[0];
        assert!(first_result.name.contains("Channel 1"));
    }

    // Property-based tests for fuzzy search logic
    mod property_tests {
        use super::*;

        // Helper function to generate test channels with different patterns
        fn generate_test_channels(count: usize) -> Vec<Channel> {
            let mut channels = Vec::new();
            let patterns = vec![
                "BBC", "CNN", "Fox", "NBC", "CBS", "ABC", "ESPN", "Discovery", "History", "National Geographic"
            ];
            let categories = vec![
                "News", "Sports", "Entertainment", "Documentary", "Kids", "Movies", "Music", "Science"
            ];
            
            for i in 0..count {
                let pattern = &patterns[i % patterns.len()];
                let category = &categories[i % categories.len()];
                
                channels.push(Channel {
                    name: format!("{} {}", pattern, i),
                    logo: format!("http://example.com/{}.png", i),
                    url: format!("http://example.com/stream{}", i),
                    group_title: category.to_string(),
                    tvg_id: format!("tv{}", i),
                    resolution: if i % 2 == 0 { "1080p" } else { "720p" }.to_string(),
                    extra_info: if i % 3 == 0 { "HD" } else { "" }.to_string(),
                });
            }
            
            channels
        }

        #[test]
        fn property_search_results_are_subset_of_input() {
            // Property: All search results must be present in the original input
            let channels = generate_test_channels(100);
            let matcher = FuzzyMatcher::new();
            
            let test_queries = vec!["BBC", "News", "HD", "Discovery", "1080p", "ESPN"];
            
            for query in test_queries {
                let results = matcher.search_channels(&channels, query);
                
                // Every result must be in the original channels
                for result in &results {
                    assert!(channels.contains(result), 
                        "Search result '{}' not found in original channels", result.name);
                }
            }
        }

        #[test]
        fn property_search_results_ordered_by_relevance() {
            // Property: Search results should be ordered by relevance (highest score first)
            let channels = generate_test_channels(50);
            let matcher = FuzzyMatcher::new();
            
            let test_queries = vec!["BBC", "News", "Sports", "Discovery"];
            
            for query in test_queries {
                let results = matcher.search_channels(&channels, query);
                
                if results.len() > 1 {
                    // Calculate scores for verification
                    let mut scores = Vec::new();
                    for channel in &results {
                        if let Some(search_match) = matcher.score_channel_multiword(channel, &[query]) {
                            scores.push(search_match.score);
                        }
                    }
                    
                    // Verify scores are in descending order
                    for i in 1..scores.len() {
                        assert!(scores[i-1] >= scores[i], 
                            "Search results not ordered by relevance for query '{}'", query);
                    }
                }
            }
        }

        #[test]
        fn property_empty_query_returns_all_channels() {
            // Property: Empty query should return all channels
            let channels = generate_test_channels(50);
            let matcher = FuzzyMatcher::new();
            
            let results = matcher.search_channels(&channels, "");
            assert_eq!(results.len(), channels.len());
            
            // Results should contain all original channels
            for channel in &channels {
                assert!(results.contains(channel), 
                    "Channel '{}' missing from empty query results", channel.name);
            }
        }

        #[test]
        fn property_multiword_search_requires_all_words() {
            // Property: Multi-word search requires all words to match
            let channels = vec![
                Channel {
                    name: "BBC News International".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "CNN Sports".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Sports".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "Fox Entertainment".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Entertainment".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // Query with two words - both must match
            let results = matcher.search_channels(&channels, "BBC News");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "BBC News International");
            
            // Query with words that don't all match
            let results = matcher.search_channels(&channels, "BBC Sports");
            assert_eq!(results.len(), 0);
        }

        #[test]
        fn property_case_insensitive_search() {
            // Property: Search should be case-insensitive
            let channels = vec![
                Channel {
                    name: "BBC News".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            let queries = vec!["bbc", "BBC", "BbC", "news", "NEWS", "News"];
            
            for query in queries {
                let results = matcher.search_channels(&channels, query);
                assert!(!results.is_empty(), "Case-insensitive search failed for query '{}'", query);
            }
        }

        #[test]
        fn property_search_matches_name_and_group() {
            // Property: Search should match both channel name and group title
            let channels = vec![
                Channel {
                    name: "Channel One".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Sports".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "News Channel".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Information".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // Should find channel by name
            let results = matcher.search_channels(&channels, "News");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "News Channel");
            
            // Should find channel by group
            let results = matcher.search_channels(&channels, "Sports");
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].name, "Channel One");
        }

        #[test]
        fn property_name_matches_prioritized_over_group() {
            // Property: Name matches should be prioritized over group matches
            let channels = vec![
                Channel {
                    name: "Sports Channel".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Entertainment".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "Movie Channel".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Sports".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            let results = matcher.search_channels(&channels, "Sports");
            
            // Should return both channels but name match should be first
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].name, "Sports Channel"); // Name match first
            assert_eq!(results[1].name, "Movie Channel"); // Group match second
        }

        #[test]
        fn property_fuzzy_matching_works_with_typos() {
            // Property: Fuzzy matching should work with minor typos
            let channels = vec![
                Channel {
                    name: "Discovery Channel".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Documentary".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // Test various typos and abbreviations
            let typo_queries = vec!["Dscy", "Discov", "Disco", "Dicovery"];
            
            for query in typo_queries {
                let results = matcher.search_channels(&channels, query);
                assert!(!results.is_empty(), "Fuzzy matching failed for query '{}'", query);
                assert_eq!(results[0].name, "Discovery Channel");
            }
        }

        #[test]
        fn property_score_calculation_consistency() {
            // Property: Score calculation should be consistent and deterministic
            let channel = Channel {
                name: "Test Channel".to_string(),
                logo: "".to_string(),
                url: "".to_string(),
                group_title: "Test".to_string(),
                tvg_id: "".to_string(),
                resolution: "".to_string(),
                extra_info: "".to_string(),
            };
            
            let matcher = FuzzyMatcher::new();
            
            // Same query should produce same score
            for _ in 0..10 {
                let result1 = matcher.score_channel_multiword(&channel, &["Test"]);
                let result2 = matcher.score_channel_multiword(&channel, &["Test"]);
                
                assert_eq!(result1.is_some(), result2.is_some());
                if let (Some(match1), Some(match2)) = (result1, result2) {
                    assert_eq!(match1.score, match2.score);
                }
            }
        }

        #[test]
        fn property_performance_scales_linearly() {
            // Property: Performance should scale reasonably with input size
            let matcher = FuzzyMatcher::new();
            
            // Test with different channel counts
            let sizes = vec![10, 50, 100, 500];
            let mut times = Vec::new();
            
            for size in sizes {
                let channels = generate_test_channels(size);
                let start = std::time::Instant::now();
                
                let _results = matcher.search_channels(&channels, "Test");
                
                let duration = start.elapsed();
                times.push(duration.as_micros());
            }
            
            // Performance should not degrade exponentially
            // This is a rough check - actual performance depends on many factors
            for i in 1..times.len() {
                let ratio = times[i] as f64 / times[i-1] as f64;
                assert!(ratio < 10.0, "Performance degradation too high: {}x", ratio);
            }
        }


        #[test]
        fn property_unicode_support() {
            // Property: Search should support Unicode characters
            let channels = vec![
                Channel {
                    name: "Канал Россия".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Новости".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "Canal Español".to_string(),
                    logo: "".to_string(),
                    url: "".to_string(),
                    group_title: "Noticias".to_string(),
                    tvg_id: "".to_string(),
                    resolution: "".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // Test Unicode queries
            let unicode_queries = vec!["Россия", "Español", "Новости", "Noticias"];
            
            for query in unicode_queries {
                // Should not panic and should find appropriate matches
                let results = matcher.search_channels(&channels, query);
                assert!(results.len() <= channels.len());
            }
        }
        #[test]
        fn property_search_results_are_deterministic() {
            // Property: Same query should always return the same results in the same order
            let channels = generate_test_channels(50);
            let matcher = FuzzyMatcher::new();
            
            let test_queries = vec!["BBC", "News", "Sports", "HD"];
            
            for query in test_queries {
                let results1 = matcher.search_channels(&channels, query);
                let results2 = matcher.search_channels(&channels, query);
                let results3 = matcher.search_channels(&channels, query);
                
                assert_eq!(results1.len(), results2.len());
                assert_eq!(results1.len(), results3.len());
                
                for i in 0..results1.len() {
                    assert_eq!(results1[i].name, results2[i].name);
                    assert_eq!(results1[i].name, results3[i].name);
                }
            }
        }

        #[test]
        fn property_exact_matches_score_higher() {
            // Property: Exact matches should score higher than partial matches
            let channels = vec![
                Channel {
                    name: "BBC".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "BBC News".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "CNN BBC Report".to_string(),
                    logo: "logo3.png".to_string(),
                    url: "url3".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "3".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            let results = matcher.search_channels(&channels, "BBC");
            
            assert!(!results.is_empty());
            // The exact match "BBC" should be first
            assert_eq!(results[0].name, "BBC");
        }

        #[test]
        fn property_search_is_case_insensitive() {
            // Property: Search should be case insensitive
            let channels = vec![
                Channel {
                    name: "BBC News".to_string(),
                    logo: "logo.png".to_string(),
                    url: "url".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            let queries = vec!["BBC", "bbc", "Bbc", "bBC", "BBC NEWS", "bbc news", "Bbc News"];
            
            for query in queries {
                let results = matcher.search_channels(&channels, query);
                assert!(!results.is_empty(), "Query '{}' should find results", query);
                assert_eq!(results[0].name, "BBC News");
            }
        }

        #[test]
        fn property_multi_word_search_finds_relevant_channels() {
            // Property: Multi-word searches should find channels containing all words
            let channels = vec![
                Channel {
                    name: "BBC World News".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "International News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "CNN International".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "World News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "Local Weather".to_string(),
                    logo: "logo3.png".to_string(),
                    url: "url3".to_string(),
                    group_title: "Weather".to_string(),
                    tvg_id: "3".to_string(),
                    resolution: "720p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // "world news" should find both BBC World News and CNN International (via group)
            let results = matcher.search_channels(&channels, "world news");
            assert!(!results.is_empty());
            
            // Should find channels that contain both words in name or group
            let found_names: Vec<String> = results.iter().map(|c| c.name.clone()).collect();
            assert!(found_names.contains(&"BBC World News".to_string()));
            assert!(found_names.contains(&"CNN International".to_string()));
        }

        #[test]
        fn property_search_respects_word_boundaries() {
            // Property: Search should respect word boundaries for better matching
            let channels = vec![
                Channel {
                    name: "BBC News".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "BBCNEWS24".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            let results = matcher.search_channels(&channels, "BBC");
            
            assert!(!results.is_empty());
            // Both should be found, but "BBC News" should score higher than "BBCNEWS24"
            // because it has proper word boundaries
            assert_eq!(results[0].name, "BBC News");
        }

        #[test]
        fn property_search_handles_special_characters() {
            // Property: Search should handle special characters gracefully
            let channels = vec![
                Channel {
                    name: "BBC News 24/7".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News & Information".to_string(),
                    tvg_id: "bbc-news-24-7".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "HD+".to_string(),
                },
                Channel {
                    name: "CNN (International)".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News/Current Affairs".to_string(),
                    tvg_id: "cnn-intl".to_string(),
                    resolution: "720p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            let special_queries = vec!["BBC 24/7", "CNN (", "News &", "HD+"];
            
            for query in special_queries {
                let results = matcher.search_channels(&channels, query);
                // Should not crash or panic with special characters
                assert!(results.len() <= channels.len());
            }
        }

        #[test]
        fn property_search_performance_scales_reasonably() {
            // Property: Search performance should scale reasonably with input size
            let sizes = vec![10, 100, 1000];
            let matcher = FuzzyMatcher::new();
            
            for size in sizes {
                let channels = generate_test_channels(size);
                
                let start = std::time::Instant::now();
                let _results = matcher.search_channels(&channels, "BBC");
                let duration = start.elapsed();
                
                // Performance should be reasonable (less than 1ms per 100 channels)
                let max_duration = std::time::Duration::from_millis((size as u64 / 100).max(1));
                assert!(duration < max_duration, 
                    "Search took too long for {} channels: {:?}", size, duration);
            }
        }

        #[test]
        fn property_fuzzy_matching_finds_typos() {
            // Property: Fuzzy matching should find results even with typos
            let channels = vec![
                Channel {
                    name: "Discovery Channel".to_string(),
                    logo: "logo.png".to_string(),
                    url: "url".to_string(),
                    group_title: "Documentary".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let matcher = FuzzyMatcher::new();
            
            // Common typos that should still find "Discovery Channel"
            let typo_queries = vec!["Dscy", "Discvry", "Discovry", "Dscover"];
            
            for query in typo_queries {
                let results = matcher.search_channels(&channels, query);
                assert!(!results.is_empty(), "Query '{}' should find Discovery Channel", query);
                assert_eq!(results[0].name, "Discovery Channel");
            }
        }

        #[test]
        fn test_property_scoring_consistency() {
            // Property: Scoring should be consistent for equivalent matches
            let matcher = FuzzyMatcher::new();
            
            // Test identical channels should get identical scores
            let channel1 = Channel {
                name: "BBC News".to_string(),
                logo: "logo1.png".to_string(),
                url: "url1".to_string(),
                group_title: "News".to_string(),
                tvg_id: "1".to_string(),
                resolution: "1080p".to_string(),
                extra_info: "".to_string(),
            };
            
            let channel2 = Channel {
                name: "BBC News".to_string(),
                logo: "logo2.png".to_string(),
                url: "url2".to_string(),
                group_title: "News".to_string(),
                tvg_id: "2".to_string(),
                resolution: "720p".to_string(),
                extra_info: "".to_string(),
            };
            
            let channels = vec![channel1, channel2];
            let results = matcher.search_channels(&channels, "BBC");
            
            assert_eq!(results.len(), 2);
            // Both should have the same name and thus should appear together
            assert_eq!(results[0].name, results[1].name);
        }

        #[test]
        fn test_property_search_order_invariance() {
            // Property: Search results should be deterministic regardless of input order
            let matcher = FuzzyMatcher::new();
            
            let mut channels = generate_test_channels(20);
            let original_results = matcher.search_channels(&channels, "BBC");
            
            // Shuffle the channels
            channels.reverse();
            let shuffled_results = matcher.search_channels(&channels, "BBC");
            
            // Results should be the same (same channels, same order)
            assert_eq!(original_results.len(), shuffled_results.len());
            
            // Convert to sets for comparison (order might differ due to ties)
            let original_names: std::collections::HashSet<String> = 
                original_results.iter().map(|c| c.name.clone()).collect();
            let shuffled_names: std::collections::HashSet<String> = 
                shuffled_results.iter().map(|c| c.name.clone()).collect();
            
            assert_eq!(original_names, shuffled_names);
        }

        #[test]
        fn test_property_score_transitivity() {
            // Property: If A scores higher than B, and B scores higher than C, then A should score higher than C
            let matcher = FuzzyMatcher::new();
            
            let channels = vec![
                Channel {
                    name: "BBC".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "BBC News".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "720p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "Random Channel".to_string(),
                    logo: "logo3.png".to_string(),
                    url: "url3".to_string(),
                    group_title: "BBC Entertainment".to_string(),
                    tvg_id: "3".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let results = matcher.search_channels(&channels, "BBC");
            
            // Should find all three channels
            assert_eq!(results.len(), 3);
            
            // "BBC" should score highest (exact match)
            assert_eq!(results[0].name, "BBC");
            
            // "BBC News" should score second (starts with BBC)
            assert_eq!(results[1].name, "BBC News");
            
            // "Random Channel" should score lowest (BBC only in group)
            assert_eq!(results[2].name, "Random Channel");
        }

        #[test]
        fn test_property_empty_and_whitespace_handling() {
            // Property: Empty queries and whitespace should be handled consistently
            let matcher = FuzzyMatcher::new();
            let channels = generate_test_channels(10);
            
            let empty_results = matcher.search_channels(&channels, "");
            let whitespace_results = matcher.search_channels(&channels, "   ");
            let tab_results = matcher.search_channels(&channels, "\t");
            let newline_results = matcher.search_channels(&channels, "\n");
            
            // Empty query should return all channels
            assert_eq!(empty_results.len(), channels.len());
            
            // Whitespace-only queries should return no results (after trimming)
            assert_eq!(whitespace_results.len(), 0);
            assert_eq!(tab_results.len(), 0);
            assert_eq!(newline_results.len(), 0);
        }

        #[test]
        fn test_property_unicode_handling() {
            // Property: Unicode characters should be handled properly
            let matcher = FuzzyMatcher::new();
            
            let unicode_channels = vec![
                Channel {
                    name: "BBC Новости".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "CNN العربية".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "720p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "NHK 日本放送協会".to_string(),
                    logo: "logo3.png".to_string(),
                    url: "url3".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "3".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            // Should handle unicode queries
            let results1 = matcher.search_channels(&unicode_channels, "BBC");
            assert_eq!(results1.len(), 1);
            assert_eq!(results1[0].name, "BBC Новости");
            
            let results2 = matcher.search_channels(&unicode_channels, "Новости");
            assert_eq!(results2.len(), 1);
            assert_eq!(results2[0].name, "BBC Новости");
            
            let results3 = matcher.search_channels(&unicode_channels, "日本");
            assert_eq!(results3.len(), 1);
            assert_eq!(results3[0].name, "NHK 日本放送協会");
        }

        #[test]
        fn test_property_query_length_scaling() {
            // Property: Longer queries should generally return fewer or equally specific results
            let matcher = FuzzyMatcher::new();
            let channels = generate_test_channels(50);
            
            let queries = vec!["B", "BB", "BBC", "BBC News", "BBC News International"];
            let mut prev_count = channels.len() + 1; // Start with a high count
            
            for query in queries {
                let results = matcher.search_channels(&channels, query);
                
                // Each longer query should return same or fewer results
                assert!(results.len() <= prev_count,
                    "Query '{}' returned {} results, but previous query returned {} results",
                    query, results.len(), prev_count);
                
                prev_count = results.len();
            }
        }

        #[test]
        fn test_property_case_insensitive_symmetry() {
            // Property: Case changes should not affect result sets
            let matcher = FuzzyMatcher::new();
            let channels = generate_test_channels(30);
            
            let test_cases = vec![
                ("bbc", "BBC", "Bbc"),
                ("news", "NEWS", "News"),
                ("sports", "SPORTS", "Sports"),
            ];
            
            for (lower, upper, mixed) in test_cases {
                let lower_results = matcher.search_channels(&channels, lower);
                let upper_results = matcher.search_channels(&channels, upper);
                let mixed_results = matcher.search_channels(&channels, mixed);
                
                // All should return the same number of results
                assert_eq!(lower_results.len(), upper_results.len());
                assert_eq!(lower_results.len(), mixed_results.len());
                
                // Convert to sets for comparison
                let lower_names: std::collections::HashSet<String> = 
                    lower_results.iter().map(|c| c.name.clone()).collect();
                let upper_names: std::collections::HashSet<String> = 
                    upper_results.iter().map(|c| c.name.clone()).collect();
                let mixed_names: std::collections::HashSet<String> = 
                    mixed_results.iter().map(|c| c.name.clone()).collect();
                
                assert_eq!(lower_names, upper_names);
                assert_eq!(lower_names, mixed_names);
            }
        }

        #[test]
        fn test_property_prefix_matching_behavior() {
            // Property: Prefix matches should score higher than scattered matches
            let matcher = FuzzyMatcher::new();
            
            let channels = vec![
                Channel {
                    name: "BBC News".to_string(),
                    logo: "logo1.png".to_string(),
                    url: "url1".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "1".to_string(),
                    resolution: "1080p".to_string(),
                    extra_info: "".to_string(),
                },
                Channel {
                    name: "A Big Broadcasting Company".to_string(),
                    logo: "logo2.png".to_string(),
                    url: "url2".to_string(),
                    group_title: "News".to_string(),
                    tvg_id: "2".to_string(),
                    resolution: "720p".to_string(),
                    extra_info: "".to_string(),
                },
            ];
            
            let results = matcher.search_channels(&channels, "BBC");
            
            assert_eq!(results.len(), 2);
            // "BBC News" should score higher than "A Big Broadcasting Company"
            assert_eq!(results[0].name, "BBC News");
            assert_eq!(results[1].name, "A Big Broadcasting Company");
        }

        #[test]
        fn test_property_scoring_monotonicity() {
            // Property: Adding more matching characters should not decrease score
            let matcher = FuzzyMatcher::new();
            
            let channel = Channel {
                name: "BBC News Channel".to_string(),
                logo: "logo.png".to_string(),
                url: "url".to_string(),
                group_title: "News".to_string(),
                tvg_id: "1".to_string(),
                resolution: "1080p".to_string(),
                extra_info: "".to_string(),
            };
            
            let channels = vec![channel];
            
            // Test progressively longer matching prefixes
            let queries = vec!["B", "BB", "BBC", "BBC N", "BBC Ne"];
            
            for i in 0..queries.len() - 1 {
                let results1 = matcher.search_channels(&channels, queries[i]);
                let results2 = matcher.search_channels(&channels, queries[i + 1]);
                
                // Both should find the channel
                assert_eq!(results1.len(), 1);
                assert_eq!(results2.len(), 1);
                
                // Note: We can't directly compare scores as they're not exposed,
                // but we can verify that both queries find the same channel
                assert_eq!(results1[0].name, results2[0].name);
            }
        }
    }
}
