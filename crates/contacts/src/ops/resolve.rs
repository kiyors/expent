use any_ascii::any_ascii;
use db::AppError;
use db::entities;
use rphonetic::{Encoder, Metaphone};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use strsim::jaro_winkler;

#[derive(Debug, Default)]
pub struct ContactResolution {
    pub contact_id: Option<String>,
    pub confidence_score: f32,
    pub collision_candidates: Vec<entities::contacts::Model>,
    pub is_collision: bool,
}

pub struct ResolveParams {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub upi_id: Option<String>,
}

const MIN_CONFIDENCE_THRESHOLD: f32 = 0.3;

/// Normalizes a name for fuzzy matching by transliterating and removing extra spaces/special chars.
/// Explicitly preserves spaces between words for better phonetic encoding.
pub(crate) fn normalize_name(name: &str) -> String {
    any_ascii(name)
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generates a phonetic representation of a name using Metaphone.
pub(crate) fn phonetic_encode(name: &str) -> String {
    let normalized = normalize_name(name);
    let metaphone = Metaphone::default();
    normalized
        .split_whitespace()
        .map(|word| metaphone.encode(word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn resolve_contact<C>(
    db: &C,
    user_id: &str,
    params: ResolveParams,
) -> Result<ContactResolution, AppError>
where
    C: ConnectionTrait,
{
    let mut matches = HashMap::new(); // contact_id -> score

    // 1. UPI Match (Weight 0.5)
    if let Some(upi) = &params.upi_id {
        let identifier = entities::contact_identifiers::Entity::find()
            .filter(entities::contact_identifiers::Column::Value.eq(upi))
            .filter(entities::contact_identifiers::Column::Type.eq("UPI"))
            .one(db)
            .await?;

        if let Some(ident) = identifier {
            let score = matches.entry(ident.contact_id.clone()).or_insert(0.0);
            *score += 0.5;
        }
    }

    // 2. Phone Match (Weight 0.3)
    if let Some(phone) = &params.phone {
        let contact = entities::contacts::Entity::find()
            .filter(entities::contacts::Column::Phone.eq(phone))
            .inner_join(entities::contact_links::Entity)
            .filter(entities::contact_links::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        if let Some(c) = contact {
            let score = matches.entry(c.id.clone()).or_insert(0.0);
            *score += 0.3;
        }
    }

    // 3. Email Match (Weight 0.1)
    if let Some(email) = &params.email {
        let identifier = entities::contact_identifiers::Entity::find()
            .filter(entities::contact_identifiers::Column::Value.eq(email))
            .filter(entities::contact_identifiers::Column::Type.eq("EMAIL"))
            .one(db)
            .await?;

        if let Some(ident) = identifier {
            let score = matches.entry(ident.contact_id.clone()).or_insert(0.0);
            *score += 0.1;
        }
    }

    // 4. Name Match (Fuzzy + Phonetic) (Weight 0.1 total)
    if let Some(name) = &params.name {
        let normalized_input = normalize_name(name);
        let phonetic_input = phonetic_encode(name);

        let contacts = entities::contacts::Entity::find()
            .inner_join(entities::contact_links::Entity)
            .filter(entities::contact_links::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        for c in contacts {
            let normalized_target = c
                .normalized_name
                .as_ref()
                .map_or_else(|| normalize_name(&c.name), |n| n.clone());
            let similarity = jaro_winkler(&normalized_input, &normalized_target) as f32;

            let mut match_score = 0.0;

            if similarity > 0.85 {
                match_score += 0.05 * similarity;
            }

            // Phonetic check
            let phonetic_target = c
                .phonetic_name
                .as_ref()
                .map_or_else(|| phonetic_encode(&c.name), |p| p.clone());
            if !phonetic_input.is_empty()
                && !phonetic_target.is_empty()
                && phonetic_input == phonetic_target
            {
                match_score += 0.05;
            }

            if match_score > 0.0 {
                let score = matches.entry(c.id.clone()).or_insert(0.0);
                *score += match_score;
            }
        }
    }

    if matches.is_empty() {
        return Ok(ContactResolution::default());
    }

    let mut sorted_matches: Vec<_> = matches.into_iter().collect();
    sorted_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let (best_contact_id, best_score) = sorted_matches[0].clone();

    // 5. Confidence Threshold Check
    if best_score < MIN_CONFIDENCE_THRESHOLD {
        return Ok(ContactResolution {
            contact_id: None,
            confidence_score: best_score,
            collision_candidates: Vec::new(),
            is_collision: false,
        });
    }

    // 6. Collision detection
    if sorted_matches.len() > 1 {
        let second_score = sorted_matches[1].1;
        // If the gap between top two matches is small, mark as collision
        if second_score > 0.25 && (best_score - second_score).abs() < 0.1 {
            let candidate_ids: Vec<String> = sorted_matches.into_iter().map(|(id, _)| id).collect();
            let candidates = entities::contacts::Entity::find()
                .filter(entities::contacts::Column::Id.is_in(candidate_ids))
                .all(db)
                .await?;

            return Ok(ContactResolution {
                contact_id: None,
                confidence_score: best_score,
                collision_candidates: candidates,
                is_collision: true,
            });
        }
    }

    Ok(ContactResolution {
        contact_id: Some(best_contact_id),
        confidence_score: best_score,
        collision_candidates: Vec::new(),
        is_collision: false,
    })
}
