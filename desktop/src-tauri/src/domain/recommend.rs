use super::media::{LibraryEntry, MediaStatus, MediaType};
use serde::Serialize;
use std::collections::BTreeMap;

const RECENT_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const RECENT_BOOST: f64 = 0.5;
const NEUTRAL_RATING: i64 = 5;
const REASON_TITLES: usize = 2;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recommendation {
    pub entry_id: String,
    pub title: String,
    pub media_type: MediaType,
    pub score: f64,
    pub reason: String,
}

/// Planned entries that share genres or tags with what the user rated well, best first.
///
/// A rating counts as `rating - 5`, so a 9 pulls its traits up and a 2 pushes them down. The
/// weights of a trait are summed and divided by the square root of how many entries carry it,
/// which lets a long history count without letting one rating decide everything.
pub fn recommend(
    entries: &[LibraryEntry],
    now: i64,
    only: Option<MediaType>,
    limit: usize,
) -> Vec<Recommendation> {
    let rated: Vec<Rated> = entries.iter().filter_map(Rated::of).collect();
    let affinity = affinities(&rated);

    let mut scored: Vec<(f64, &LibraryEntry, String)> = entries
        .iter()
        .filter(|entry| entry.status == MediaStatus::Planned)
        .filter(|entry| only.is_none_or(|kind| kind == entry.media_type))
        .filter_map(|entry| {
            let (base, reason) = judge(entry, &affinity, &rated)?;
            let boost = if now - entry.created_at <= RECENT_MS {
                RECENT_BOOST
            } else {
                0.0
            };
            Some((base + boost, entry, reason))
        })
        .collect();

    scored.sort_by(|(a, left, _), (b, right, _)| {
        b.total_cmp(a)
            .then(right.created_at.cmp(&left.created_at))
            .then(left.title.cmp(&right.title))
            .then(left.id.cmp(&right.id))
    });

    scored
        .into_iter()
        .take(limit)
        .map(|(score, entry, reason)| Recommendation {
            entry_id: entry.id.clone(),
            title: entry.title.clone(),
            media_type: entry.media_type,
            score: (score * 100.0).round() / 100.0,
            reason,
        })
        .collect()
}

/// An entry the user rated, with how far the rating is from neutral.
struct Rated<'a> {
    entry: &'a LibraryEntry,
    weight: i64,
    traits: Vec<String>,
}

impl<'a> Rated<'a> {
    fn of(entry: &'a LibraryEntry) -> Option<Self> {
        if entry.status == MediaStatus::Planned {
            return None;
        }
        Some(Self {
            entry,
            weight: entry.rating? - NEUTRAL_RATING,
            traits: traits(entry),
        })
    }
}

/// Lowercase genres and tags without duplicates, in a fixed order.
fn traits(entry: &LibraryEntry) -> Vec<String> {
    let mut all: Vec<String> = entry
        .genres
        .0
        .iter()
        .chain(entry.tags.0.iter())
        .map(|name| name.trim().to_lowercase())
        .filter(|name| !name.is_empty())
        .collect();
    all.sort();
    all.dedup();
    all
}

fn affinities(rated: &[Rated]) -> BTreeMap<String, f64> {
    let mut sums: BTreeMap<String, (i64, usize)> = BTreeMap::new();
    for item in rated {
        for name in &item.traits {
            let slot = sums.entry(name.clone()).or_default();
            slot.0 += item.weight;
            slot.1 += 1;
        }
    }
    sums.into_iter()
        .map(|(name, (sum, count))| (name, sum as f64 / (count as f64).sqrt()))
        .collect()
}

/// The score before any boost and the sentence that explains it; `None` when nothing speaks for the entry.
fn judge(
    entry: &LibraryEntry,
    affinity: &BTreeMap<String, f64>,
    rated: &[Rated],
) -> Option<(f64, String)> {
    let own = traits(entry);
    let known: Vec<(&String, f64)> = own
        .iter()
        .filter_map(|name| Some((name, *affinity.get(name.as_str())?)))
        .collect();
    let base = known.iter().map(|(_, value)| value).sum::<f64>() / (own.len() as f64).sqrt();
    if own.is_empty() || base <= 0.0 {
        return None;
    }

    // the first of the strongest traits wins a tie, and the traits are sorted
    let (best, _) = known
        .iter()
        .copied()
        .reduce(|top, next| if next.1 > top.1 { next } else { top })?;
    let mut liked: Vec<&Rated> = rated
        .iter()
        .filter(|item| item.weight > 0 && item.traits.contains(best))
        .collect();
    liked.sort_by(|a, b| {
        b.weight
            .cmp(&a.weight)
            .then(a.entry.title.cmp(&b.entry.title))
            .then(a.entry.id.cmp(&b.entry.id))
    });
    let titles: Vec<&str> = liked
        .iter()
        .take(REASON_TITLES)
        .map(|item| item.entry.title.as_str())
        .collect();
    Some((
        base,
        format!("Because you liked {} ({best})", titles.join(" and ")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::media::MediaSource;
    use sqlx::types::Json;

    const DAY: i64 = 24 * 60 * 60 * 1000;
    const NOW: i64 = 100 * DAY;

    struct Spec {
        title: &'static str,
        status: MediaStatus,
        rating: Option<i64>,
        genres: &'static [&'static str],
        age_days: i64,
    }

    fn done(title: &'static str, rating: i64, genres: &'static [&'static str]) -> Spec {
        Spec {
            title,
            status: MediaStatus::Completed,
            rating: Some(rating),
            genres,
            age_days: 90,
        }
    }

    fn planned(title: &'static str, genres: &'static [&'static str], age_days: i64) -> Spec {
        Spec {
            title,
            status: MediaStatus::Planned,
            rating: None,
            genres,
            age_days,
        }
    }

    fn entry(spec: &Spec) -> LibraryEntry {
        LibraryEntry {
            id: format!("id-{}", spec.title),
            created_at: NOW - spec.age_days * DAY,
            updated_at: 0,
            media_item_id: format!("item-{}", spec.title),
            media_type: MediaType::Anime,
            title: spec.title.to_string(),
            cover_url: None,
            cover_cache_path: None,
            cover_path: None,
            year: None,
            genres: Json(spec.genres.iter().map(|g| g.to_string()).collect()),
            synopsis: String::new(),
            source: MediaSource::Manual,
            external_id: None,
            status: spec.status,
            progress: 0,
            progress_total: None,
            rating: spec.rating,
            notes: String::new(),
            tags: Json(Vec::new()),
            started_at: None,
            finished_at: None,
            platform: None,
        }
    }

    fn titles(specs: &[Spec], only: Option<MediaType>) -> Vec<String> {
        let entries: Vec<LibraryEntry> = specs.iter().map(entry).collect();
        recommend(&entries, NOW, only, 10)
            .into_iter()
            .map(|r| r.title)
            .collect()
    }

    #[test]
    fn loved_genres_rank_first_and_disliked_ones_are_left_out() {
        let specs = [
            done("Loved action", 10, &["action"]),
            done("Hated romance", 2, &["romance"]),
            planned("Action plan", &["action"], 90),
            planned("Romance plan", &["romance"], 90),
            planned("Unknown plan", &["sports"], 90),
        ];

        assert_eq!(titles(&specs, None), ["Action plan"]);
    }

    #[test]
    fn without_ratings_nothing_is_recommended() {
        let specs = [
            Spec {
                rating: None,
                ..done("Unrated", 0, &["action"])
            },
            planned("Plan", &["action"], 90),
        ];

        assert!(titles(&specs, None).is_empty());
    }

    #[test]
    fn only_planned_entries_are_candidates() {
        let mut watching = planned("Watching", &["action"], 90);
        watching.status = MediaStatus::InProgress;
        let specs = [done("Loved", 9, &["action"]), watching];

        assert!(titles(&specs, None).is_empty());
    }

    #[test]
    fn more_shared_liked_genres_score_higher() {
        let specs = [
            done("A", 9, &["action"]),
            done("B", 9, &["comedy"]),
            planned("One match", &["action", "sports"], 90),
            planned("Two matches", &["action", "comedy"], 90),
        ];

        assert_eq!(titles(&specs, None), ["Two matches", "One match"]);
    }

    #[test]
    fn a_long_history_counts_more_than_one_rating_but_not_linearly() {
        let specs = [
            done("A1", 8, &["action"]),
            done("A2", 8, &["action"]),
            done("A3", 8, &["action"]),
            done("B1", 10, &["drama"]),
            planned("Action plan", &["action"], 90),
            planned("Drama plan", &["drama"], 90),
        ];

        // action: 9 / sqrt(3) = 5.2, drama: 5 / 1 = 5.0
        assert_eq!(titles(&specs, None), ["Action plan", "Drama plan"]);
    }

    #[test]
    fn a_recent_plan_gets_a_small_boost() {
        let specs = [
            done("Loved", 9, &["action"]),
            planned("Old plan", &["action"], 90),
            planned("New plan", &["action"], 3),
        ];
        let entries: Vec<LibraryEntry> = specs.iter().map(entry).collect();

        let result = recommend(&entries, NOW, None, 10);

        assert_eq!(result[0].title, "New plan");
        assert_eq!(result[0].score - result[1].score, RECENT_BOOST);
    }

    #[test]
    fn equal_scores_keep_a_stable_order() {
        let specs = [
            done("Loved", 9, &["action"]),
            planned("Zeta", &["action"], 90),
            planned("Alpha", &["action"], 90),
            planned("Newer", &["action"], 80),
        ];

        // the newer entry first, then by title
        assert_eq!(titles(&specs, None), ["Newer", "Alpha", "Zeta"]);
        assert_eq!(titles(&specs, None), titles(&specs, None));
    }

    #[test]
    fn the_reason_names_the_best_rated_matches() {
        let specs = [
            done("Okay", 6, &["action"]),
            done("Great", 10, &["action"]),
            done("Good", 8, &["action"]),
            planned("Plan", &["action"], 90),
        ];
        let entries: Vec<LibraryEntry> = specs.iter().map(entry).collect();

        let result = recommend(&entries, NOW, None, 10);

        assert_eq!(
            result[0].reason,
            "Because you liked Great and Good (action)"
        );
    }

    #[test]
    fn genres_match_without_regard_to_case_and_spaces() {
        let specs = [
            done("Loved", 9, &[" Action "]),
            planned("Plan", &["action"], 90),
        ];

        assert_eq!(titles(&specs, None), ["Plan"]);
    }

    #[test]
    fn tags_count_as_traits() {
        let mut loved = entry(&done("Loved", 9, &[]));
        loved.tags = Json(vec!["cozy".to_string()]);
        let mut plan = entry(&planned("Plan", &[], 90));
        plan.tags = Json(vec!["Cozy".to_string()]);

        let result = recommend(&[loved, plan], NOW, None, 10);

        assert_eq!(result[0].reason, "Because you liked Loved (cozy)");
    }

    #[test]
    fn the_type_filter_and_the_limit_apply() {
        let mut game = entry(&planned("Game plan", &["action"], 90));
        game.media_type = MediaType::Game;
        let entries = [
            entry(&done("Loved", 9, &["action"])),
            entry(&planned("Anime plan", &["action"], 90)),
            game,
        ];

        let only_games = recommend(&entries, NOW, Some(MediaType::Game), 10);
        let one = recommend(&entries, NOW, None, 1);

        assert_eq!(only_games.len(), 1);
        assert_eq!(only_games[0].title, "Game plan");
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn entries_without_genres_or_tags_are_skipped() {
        let specs = [done("Loved", 9, &["action"]), planned("Bare", &[], 90)];

        assert!(titles(&specs, None).is_empty());
    }
}
