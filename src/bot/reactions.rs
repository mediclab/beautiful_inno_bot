use crate::db::entity::prelude::{Photos, Reactions};
use std::collections::HashMap;
use teloxide::dispatching::{UpdateFilterExt, UpdateHandler};
use teloxide::prelude::Update;
use teloxide::types::{MessageReactionCountUpdated, ReactionType};
use uuid::Uuid;

#[tracing::instrument(skip_all)]
pub async fn handle_reactions_count(react: MessageReactionCountUpdated) -> anyhow::Result<()> {
    debug!("Received reaction count updated: {:?}", &react);

    if let Some(photo) = Photos::get_by_channel_msg_id(react.message_id.0).await {
        let reactions_from: Vec<ReactionType> = react.reactions.iter().map(|r| r.r#type.clone()).collect();
        let reactions: HashMap<Uuid, ReactionType> = photo.get_reactions().await.into_iter().map(|r| (r.uuid, r.into())).collect();
        let for_delete: Vec<Uuid> = reactions.iter().filter(|r| !reactions_from.contains(r.1)).map(|r| r.0).cloned().collect();

        if !for_delete.is_empty() {
            Reactions::remove_reactions(for_delete).await;
        }

        if !reactions_from.is_empty() {
            Reactions::update_reactions(photo.uuid, react.reactions.iter().map(|r| r.into()).collect()).await;
        }
    }
    Ok(())
}

pub fn scheme() -> UpdateHandler<anyhow::Error> {
    teloxide::dptree::entry().branch(Update::filter_message_reaction_count_updated().endpoint(handle_reactions_count))
}
