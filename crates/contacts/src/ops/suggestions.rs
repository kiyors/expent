use db::AppError;
use db::entities;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strsim::jaro_winkler;

#[derive(Serialize, Deserialize, Debug)]
pub struct MergeSuggestion {
    pub contacts: Vec<entities::contacts::Model>,
    pub reason: String,
}

// Read similarity threshold from env, default to 0.88 if not provided or invalid
fn get_similarity_threshold() -> f64 {
    std::env::var("CONTACT_MERGE_SIMILARITY_THRESHOLD")
        .unwrap_or_else(|_| "0.88".to_string())
        .parse()
        .unwrap_or(0.88)
}

pub async fn get_merge_suggestions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<MergeSuggestion>, AppError> {
    // 1. Fetch user's contact links
    let links = entities::contact_links::Entity::find()
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    if links.len() < 2 {
        return Ok(vec![]);
    }

    let contact_ids: Vec<String> = links.into_iter().map(|link| link.contact_id).collect();

    // 2. Fetch the actual contacts
    let contacts = entities::contacts::Entity::find()
        .filter(entities::contacts::Column::Id.is_in(contact_ids.clone()))
        .all(db)
        .await?;

    // 3. Fetch identifiers for these contacts
    let identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.is_in(contact_ids))
        .all(db)
        .await?;

    // Pre-process identifiers into a HashMap for O(1) average-case lookups
    let mut identifiers_map: HashMap<String, Vec<entities::contact_identifiers::Model>> =
        HashMap::with_capacity(identifiers.len());
    for id in identifiers {
        identifiers_map
            .entry(id.contact_id.clone())
            .or_default()
            .push(id);
    }

    #[allow(clippy::items_after_statements)]
    struct CachedContact<'a> {
        contact: &'a entities::contacts::Model,
        lower_name: String,
    }

    let cached_contacts: Vec<CachedContact<'_>> = contacts
        .iter()
        .map(|c| CachedContact {
            contact: c,
            lower_name: c.name.to_lowercase(),
        })
        .collect();

    let mut suggestions: Vec<MergeSuggestion> = Vec::new();
    let similarity_threshold = get_similarity_threshold();

    for (i, c1) in cached_contacts.iter().enumerate() {
        for c2 in cached_contacts.iter().skip(i + 1) {
            let mut match_reason: Option<String> = None;

            // 1. Check exact phone match
            if let (Some(p1), Some(p2)) = (&c1.contact.phone, &c2.contact.phone)
                && !p1.trim().is_empty()
                && p1 == p2
            {
                match_reason = Some("Same phone number".to_string());
            }

            // 2. Check identifier overlap (UPI, Bank Acc)
            if match_reason.is_none() {
                let empty_vec: Vec<entities::contact_identifiers::Model> = Vec::new();
                let id1s = identifiers_map
                    .get(c1.contact.id.as_str())
                    .unwrap_or(&empty_vec);
                let id2s = identifiers_map
                    .get(c2.contact.id.as_str())
                    .unwrap_or(&empty_vec);

                'outer: for id1 in id1s {
                    for id2 in id2s {
                        if id1.r#type == id2.r#type && id1.value == id2.value {
                            match_reason = Some(format!("Shared {} identifier", id1.r#type));
                            break 'outer;
                        }
                    }
                }
            }

            // 3. Check fuzzy name match
            if match_reason.is_none()
                && jaro_winkler(&c1.lower_name, &c2.lower_name) > similarity_threshold
            {
                match_reason = Some("Similar name".to_string());
            }

            if let Some(reason) = match_reason {
                suggestions.push(MergeSuggestion {
                    contacts: vec![c1.contact.clone(), c2.contact.clone()],
                    reason,
                });
            }
        }
    }

    Ok(suggestions)
}
