#[macro_use]
extern crate log;
#[macro_use]
extern crate rust_i18n;

extern crate core;
extern crate inflector;

use crate::bot::{BotConfig, BotManager};
use crate::redis::{RedisConfig, RedisManager};
use dotenv::dotenv;
use envconfig::Envconfig;
use std::sync::Arc;
use teloxide::{
    dispatching::dialogue::{RedisStorage, serializer::Json},
    prelude::*,
};

mod bot;
mod db;
mod exif;
mod image;
mod redis;
mod types;

#[derive(Clone)]
pub struct Application {
    config: Config,
}

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub db_url: String,
    #[envconfig(from = "BOT_VERSION", default = "unknown")]
    pub version: String,
    #[envconfig(nested)]
    pub bot_config: BotConfig,
    #[envconfig(nested)]
    pub redis_config: RedisConfig,
}

impl Application {
    pub fn new() -> Self {
        Self {
            config: Config::init_from_env().expect("Can't load config"),
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}

i18n!("locales", fallback = "ru");

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init_timed();

    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    });

    let app = Arc::new(Application::new());

    let db = db::Database::new(&app.config.db_url).await;
    let bot = bot::BotManager::new(&app.config.bot_config);
    let redis = RedisManager::new(&app.config.redis_config);
    db.migrate().await.expect("Can't migrate");

    db::INSTANCE.set(db).expect("Can't set database");
    bot::INSTANCE.set(bot).expect("Can't set bot");
    redis::INSTANCE.set(redis).expect("Can't set redis");

    info!("Bot version: {}", &app.config.version);

    info!("Starting subscriber...");
    RedisManager::global().subscriber(&app.config.bot_config).await;

    info!("Starting dispatch...");
    BotManager::global()
        .dispatch(dptree::deps![
            Arc::clone(&app),
            RedisStorage::open(&app.config.redis_config.url, Json)
                .await
                .expect("Can't connect dialogues on redis")
        ])
        .await;

    info!("Good Bye!");
}
