use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{path::Path, thread::sleep, time::Duration};
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching::{
        Dispatcher,
        dialogue::{RedisStorage, serializer::Json},
    },
    dptree,
    net::Download,
    prelude::*,
    types::ParseMode,
};
use tokio::fs::File;

mod callback;
mod command;
mod dialogue;
pub(super) mod markups;
mod message;
mod reactions;
pub(super) mod traits;
pub(super) mod types;

pub static INSTANCE: OnceCell<BotManager> = OnceCell::new();

pub type Bot = DefaultParseMode<teloxide::Bot>;
pub type BotDialogue = Dialogue<GlobalState, RedisStorage<Json>>;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum GlobalState {
    #[default]
    Idle,
    DeclinePhoto(dialogue::decline_photo::State),
    BanUser(dialogue::ban_user::State),
}

#[derive(Envconfig, Clone, Debug)]
pub struct BotConfig {
    #[envconfig(from = "GROUP_ID")]
    pub group_id: i64,
    #[envconfig(from = "ADMIN_USER_ID")]
    pub admin_id: i64,
    #[envconfig(from = "BOT_TOKEN")]
    pub bot_token: String,
}

#[derive(Clone, Debug)]
pub struct BotManager {
    bot: Bot,
    group_id: i64,
    admin_id: i64,
}

impl BotManager {
    pub fn new(config: &BotConfig) -> Self {
        Self {
            bot: teloxide::Bot::new(&config.bot_token).parse_mode(ParseMode::Html),
            admin_id: config.admin_id,
            group_id: config.group_id,
        }
    }

    pub fn global() -> &'static BotManager {
        INSTANCE.get().expect("BotManager is not initialized")
    }

    pub fn get_admin_id(&self) -> i64 {
        self.admin_id
    }

    pub fn get_group_id(&self) -> i64 {
        self.group_id
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub async fn dispatch(&self, deps: DependencyMap) {
        Dispatcher::builder(
            self.bot.clone(),
            dptree::entry()
                .branch(dialogue::scheme())
                .branch(command::scheme())
                .branch(message::scheme())
                .branch(reactions::scheme())
                .branch(callback::scheme()),
        )
        .dependencies(deps)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await
    }

    pub async fn download_doc(&self, doc_id: &str, save_path: &Path) -> anyhow::Result<String> {
        let doc = self.bot.get_file(doc_id.to_owned().into()).await?;
        let mut file = File::create(&save_path).await?;

        self.bot.download_file(&doc.path, &mut file).await?;
        sleep(Duration::from_millis(50)); // Sometimes downloading is very fast
        debug!("Filesize {} is = {}", save_path.to_str().unwrap(), std::fs::metadata(save_path)?.len());

        Ok(save_path.to_str().unwrap().to_string())
    }
}
