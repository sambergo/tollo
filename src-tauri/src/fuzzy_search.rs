use crate::m3u_parser::Channel;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub channel: Channel,
    pub score: i32,
    pub match_positions: Vec<usize>,
}

pub struct FuzzyMatcher {
    case_sensitive: bool,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
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
