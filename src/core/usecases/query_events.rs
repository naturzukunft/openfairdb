use super::EventQuery;
use crate::core::{
    prelude::*,
    util::{extract_hash_tags, filter, remove_hash_tags},
};

const DEFAULT_RESULT_LIMIT: usize = 100;
const MAX_RESULT_LIMIT: usize = 500;

#[allow(clippy::absurd_extreme_comparisons)]
pub fn query_events<D: Db>(
    db: &D,
    index: &dyn IdIndex,
    query: EventQuery,
    token: Option<String>,
) -> Result<Vec<Event>> {
    if query.is_empty() {
        // Special case for backwards compatibility
        return Ok(db.all_events_chronologically()?);
    }
    let EventQuery {
        bbox: visible_bbox,
        created_by,
        start_min,
        start_max,
        tags,
        text,
        limit,
    } = query;
    let _org = if let Some(ref token) = token {
        let org = db.get_org_by_api_token(token).map_err(|e| match e {
            RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
            _ => Error::Repo(e),
        })?;
        Some(org)
    } else {
        None
    };

    let mut hash_tags = text
        .as_ref()
        .map(String::as_str)
        .map(extract_hash_tags)
        .unwrap_or_default();
    if let Some(tags) = tags {
        hash_tags.reserve(hash_tags.len() + tags.len());
        for hashtag in tags {
            hash_tags.push(hashtag.to_owned());
        }
    }

    let text = text
        .as_ref()
        .map(String::as_str)
        .map(remove_hash_tags)
        .and_then(|text| {
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        });

    let text_tags = text
        .as_ref()
        .map(String::as_str)
        .map(filter::split_text_to_words)
        .unwrap_or_default();

    let visible_events_query = IndexQuery {
        include_bbox: visible_bbox,
        exclude_bbox: None,
        categories: vec![Category::ID_EVENT],
        hash_tags,
        text_tags,
        text,
        ts_min_lb: start_min,
        ts_min_ub: start_max,
        ..Default::default()
    };

    let search_limit = if let Some(limit) = limit {
        if limit > MAX_RESULT_LIMIT {
            info!(
                "Requested limit {} exceeds maximum limit {} for search results",
                limit, MAX_RESULT_LIMIT
            );
            MAX_RESULT_LIMIT
        } else if limit <= 0 {
            warn!("Invalid search limit: {}", limit);
            return Err(Error::Parameter(ParameterError::InvalidLimit));
        } else {
            limit
        }
    } else {
        info!(
            "No limit requested - Using default limit {} for search results",
            DEFAULT_RESULT_LIMIT
        );
        DEFAULT_RESULT_LIMIT
    };

    // 1st query: Search for visible results only
    // This is required to reliably retrieve all available results!
    // See also: https://github.com/slowtec/openfairdb/issues/183
    let visible_event_ids = index
        .query_ids(&visible_events_query, search_limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;

    // 2nd query: Search for remaining invisible results
    let invisible_event_ids = if let Some(visible_bbox) = visible_bbox {
        if visible_event_ids.len() < search_limit {
            let invisible_events_query = IndexQuery {
                include_bbox: Some(filter::extend_bbox(&visible_bbox)),
                exclude_bbox: visible_events_query.include_bbox,
                ..visible_events_query
            };
            index
                .query_ids(
                    &invisible_events_query,
                    search_limit - visible_event_ids.len(),
                )
                .map_err(|err| RepoError::Other(Box::new(err.compat())))?
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let event_ids: Vec<_> = visible_event_ids
        .iter()
        .chain(invisible_event_ids.iter())
        .map(Id::as_str)
        .collect();
    let mut events = db.get_events(&event_ids)?;

    if let Some(ref email) = created_by {
        if let Some(user) = db.try_get_user_by_email(email)? {
            events = events
                .into_iter()
                .filter(|e| e.created_by.as_ref() == Some(&user.email))
                .collect();
        } else {
            events = vec![];
        }
    }

    events.sort_by(|a, b| a.start.cmp(&b.start));

    Ok(events)
}
