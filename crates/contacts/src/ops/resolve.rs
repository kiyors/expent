use any_ascii::any_ascii;
use db::AppError;
use db::entities;
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
const COLLISION_SCORE_DIFFERENCE_THRESHOLD: f64 = 0.1;
const FUZZY_MATCH_THRESHOLD: f64 = 0.85;
const FUZZY_MATCH_SCORE_INCREMENT: f64 = 0.05;
const PHONETIC_MATCH_SCORE_INCREMENT: f64 = 0.05;

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
    use rphonetic::{Encoder, Metaphone};
    let normalized = normalize_name(name);
    let metaphone = Metaphone::default();
    normalized
        .split_whitespace()
        .map(|word| metaphone.encode(word))
        .collect::<Vec<_>>()
        .join(" ")
}

async fn get_upi_match<C>(db: &C, upi_id: &str) -> Result<Option<String>, AppError>
where
    C: ConnectionTrait,
{
    let identifier = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::Value.eq(upi_id))
        .filter(entities::contact_identifiers::Column::Type.eq("UPI"))
        .one(db)
        .await?;
    Ok(identifier.map(|ident| ident.contact_id))
}

async fn get_phone_match<C>(db: &C, user_id: &str, phone: &str) -> Result<Option<String>, AppError>
where
    C: ConnectionTrait,
{
    let contact = entities::contacts::Entity::find()
        .filter(entities::contacts::Column::Phone.eq(phone))
        .inner_join(entities::contact_links::Entity)
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .one(db)
        .await?;
    Ok(contact.map(|c| c.id))
}

async fn get_email_match<C>(db: &C, email: &str) -> Result<Option<String>, AppError>
where
    C: ConnectionTrait,
{
    let identifier = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::Value.eq(email))
        .filter(entities::contact_identifiers::Column::Type.eq("EMAIL"))
        .one(db)
        .await?;
    Ok(identifier.map(|ident| ident.contact_id))
}

pub async fn resolve_contacts_bulk<C>(
    db: &C,
    user_id: &str,
    batch: Vec<ResolveParams>,
) -> Result<Vec<ContactResolution>, AppError>
where
    C: ConnectionTrait,
{
    if batch.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Fetch all user contacts once for fuzzy matching
    let all_contacts = entities::contacts::Entity::find()
        .inner_join(entities::contact_links::Entity)
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    // 2. Pre-fetch relevant identifiers for bulk lookups
    let upi_ids: Vec<String> = batch.iter().filter_map(|p| p.upi_id.clone()).collect();
    let upi_map: HashMap<String, String> = if upi_ids.is_empty() {
        HashMap::new()
    } else {
        entities::contact_identifiers::Entity::find()
            .filter(entities::contact_identifiers::Column::Type.eq("UPI"))
            .filter(entities::contact_identifiers::Column::Value.is_in(upi_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|i| (i.value, i.contact_id))
            .collect()
    };

    let phones: Vec<String> = batch.iter().filter_map(|p| p.phone.clone()).collect();
    let phone_map: HashMap<String, String> = if phones.is_empty() {
        HashMap::new()
    } else {
        entities::contacts::Entity::find()
            .filter(entities::contacts::Column::Phone.is_in(phones))
            .inner_join(entities::contact_links::Entity)
            .filter(entities::contact_links::Column::UserId.eq(user_id))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|c| c.phone.map(|p| (p, c.id)))
            .collect()
    };

    let mut results = Vec::with_capacity(batch.len());

    for params in batch {
        let mut matches: HashMap<String, f64> = HashMap::new();

        if let Some(upi_id) = &params.upi_id {
            if let Some(id) = upi_map.get(upi_id) {
                *matches.entry(id.clone()).or_insert(0.0) += 0.5;
            }
        }

        if let Some(phone) = &params.phone {
            if let Some(id) = phone_map.get(phone) {
                *matches.entry(id.clone()).or_insert(0.0) += 0.3;
            }
        }

        if let Some(name) = &params.name {
            let normalized_input = normalize_name(name);
            let phonetic_input = phonetic_encode(name);

            for c in &all_contacts {
                let mut match_score = 0.0;

                let normalized_target = c.normalized_name.as_deref().map_or_else(
                    || std::borrow::Cow::Owned(normalize_name(&c.name)),
                    std::borrow::Cow::Borrowed,
                );

                let similarity = jaro_winkler(&normalized_input, &normalized_target);
                if similarity > FUZZY_MATCH_THRESHOLD {
                    match_score += FUZZY_MATCH_SCORE_INCREMENT * similarity;
                }

                let phonetic_target = c.phonetic_name.as_deref().map_or_else(
                    || std::borrow::Cow::Owned(phonetic_encode(&c.name)),
                    std::borrow::Cow::Borrowed,
                );

                if !phonetic_input.is_empty()
                    && !phonetic_target.is_empty()
                    && phonetic_input == *phonetic_target
                {
                    match_score += PHONETIC_MATCH_SCORE_INCREMENT;
                }

                if match_score > 0.0 {
                    *matches.entry(c.id.clone()).or_insert(0.0) += match_score;
                }
            }
        }

        if matches.is_empty() {
            results.push(ContactResolution::default());
            continue;
        }

        let mut sorted_matches: Vec<(String, f64)> = matches.into_iter().collect();
        sorted_matches.sort_by(|a, b| b.1.total_cmp(&a.1));

        let (best_contact_id, best_score) = sorted_matches[0].clone();

        if best_score < f64::from(MIN_CONFIDENCE_THRESHOLD) {
            results.push(ContactResolution {
                contact_id: None,
                confidence_score: best_score as f32,
                ..Default::default()
            });
            continue;
        }

        // Collision detection (simplified for bulk)
        if sorted_matches.len() > 1 {
            let second_score = sorted_matches[1].1;
            if second_score > 0.25
                && (best_score - second_score).abs() < COLLISION_SCORE_DIFFERENCE_THRESHOLD
            {
                results.push(ContactResolution {
                    contact_id: None,
                    confidence_score: best_score as f32,
                    is_collision: true,
                    // Note: collision_candidates omitted for performance in bulk
                    ..Default::default()
                });
                continue;
            }
        }

        results.push(ContactResolution {
            contact_id: Some(best_contact_id),
            confidence_score: best_score as f32,
            ..Default::default()
        });
    }

    Ok(results)
}

#[allow(clippy::too_many_lines)]
/// Attempts to resolve a contact based on provided identifiers (UPI, Phone, Email) and name.
///
/// # Errors
/// Returns `AppError::Db` if any database query fails.
pub async fn resolve_contact<C>(
    db: &C,
    user_id: &str,
    params: ResolveParams,
) -> Result<ContactResolution, AppError>
where
    C: ConnectionTrait,
{
    let mut matches: HashMap<String, f64> = HashMap::new(); // contact_id -> score

    if let Some(upi_id) = params.upi_id
        && let Some(id) = get_upi_match(db, &upi_id).await?
    {
        *matches.entry(id).or_insert(0.0) += 0.5; // Weight 0.5
    }

    if let Some(phone) = params.phone
        && let Some(id) = get_phone_match(db, user_id, &phone).await?
    {
        *matches.entry(id).or_insert(0.0) += 0.3; // Weight 0.3
    }

    if let Some(email) = params.email
        && let Some(id) = get_email_match(db, &email).await?
    {
        *matches.entry(id).or_insert(0.0) += 0.1; // Weight 0.1
    }
    if let Some(name) = params.name {
        let normalized_input = normalize_name(&name);
        let phonetic_input = phonetic_encode(&name);

        let contacts = entities::contacts::Entity::find()
            .inner_join(entities::contact_links::Entity)
            .filter(entities::contact_links::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        for c in contacts {
            let normalized_target = c.normalized_name.as_deref().map_or_else(
                || std::borrow::Cow::Owned(normalize_name(&c.name)),
                std::borrow::Cow::Borrowed,
            );

            let similarity = jaro_winkler(&normalized_input, &normalized_target);

            let mut match_score = 0.0;

            if similarity > FUZZY_MATCH_THRESHOLD {
                match_score += FUZZY_MATCH_SCORE_INCREMENT * similarity;
            }

            // Phonetic check
            let phonetic_target = c.phonetic_name.as_deref().map_or_else(
                || std::borrow::Cow::Owned(phonetic_encode(&c.name)),
                std::borrow::Cow::Borrowed,
            );

            if !phonetic_input.is_empty()
                && !phonetic_target.is_empty()
                && phonetic_input == *phonetic_target
            {
                match_score += PHONETIC_MATCH_SCORE_INCREMENT;
            }

            if match_score > 0.0 {
                *matches.entry(c.id).or_insert(0.0) += match_score;
            }
        }
    }

    if matches.is_empty() {
        return Ok(ContactResolution::default());
    }

    let mut sorted_matches: Vec<(String, f64)> = matches.into_iter().collect();
    // Use total_cmp for safe f64 sorting
    sorted_matches.sort_by(|a, b| b.1.total_cmp(&a.1));

    let (best_contact_id, best_score) = sorted_matches[0].clone();

    #[allow(clippy::cast_possible_truncation)]
    if best_score < f64::from(MIN_CONFIDENCE_THRESHOLD) {
        return Ok(ContactResolution {
            contact_id: None,
            confidence_score: best_score as f32,
            collision_candidates: Vec::new(),
            is_collision: false,
        });
    }

    // Collision detection
    if sorted_matches.len() > 1 {
        let second_score = sorted_matches[1].1;
        // If the gap between top two matches is small, mark as collision
        if second_score > 0.25
            && (best_score - second_score).abs() < COLLISION_SCORE_DIFFERENCE_THRESHOLD
        {
            let candidate_ids: Vec<String> = sorted_matches.into_iter().map(|(id, _)| id).collect();
            let candidates = entities::contacts::Entity::find()
                .filter(entities::contacts::Column::Id.is_in(candidate_ids))
                .all(db)
                .await?;

            return Ok(ContactResolution {
                contact_id: None,
                #[allow(clippy::cast_possible_truncation)]
                confidence_score: best_score as f32,
                collision_candidates: candidates,
                is_collision: true,
            });
        }
    }

    Ok(ContactResolution {
        contact_id: Some(best_contact_id),
        #[allow(clippy::cast_possible_truncation)]
        confidence_score: best_score as f32,
        collision_candidates: Vec::new(),
        is_collision: false,
    })
}
