use crate::redis::RedisManager;
use serde::{Serialize, de::DeserializeOwned};

pub trait DialogueContext: Serialize + DeserializeOwned {
    #[tracing::instrument(skip_all)]
    async fn get(user_id: i64) -> Option<Self> {
        let redis = RedisManager::global();
        let name = std::any::type_name::<Self>();

        redis.get_model(&format!("{user_id}_{name}")).await
    }

    #[tracing::instrument(skip_all)]
    async fn set(&self, user_id: i64) -> bool {
        let redis = RedisManager::global();
        let name = std::any::type_name::<Self>();

        redis.set_model(&format!("{user_id}_{name}"), self).await
    }
}
