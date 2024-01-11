#[macro_use]
extern crate log;

use crate::bot::{BotCommand, CallbackHandler, CommandHandler, MessageHandler};
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::adaptors::DefaultParseMode;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

mod bot;
mod exif;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Application {
    bot: DefaultParseMode<Bot>,
    admin: i64,
    group_id: i64,
    version: String,
}

impl Application {
    pub fn new() -> Self {
        Self {
            bot: Bot::from_env().parse_mode(ParseMode::Html),
            admin: std::env::var("ADMIN_USER_ID")
                .expect("Необходимо указать получателя, кому отправлять все кубы!")
                .parse::<i64>()
                .expect("Неверный ID поллучателя кубов!"),
            group_id: std::env::var("GROUP_ID")
                .expect("Необходимо указать группу в которую бот будет постить фото!")
                .parse::<i64>()
                .expect("Неверный ID группы!"),
            version: VERSION.to_string(),
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init_timed();

    info!("Bot version: {}", VERSION);

    let app = Arc::new(Application::new());

    info!("Starting dispatch...");

    Dispatcher::builder(
        app.bot.clone(),
        dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<BotCommand>()
                    .endpoint(CommandHandler::handle),
            )
            .branch(Update::filter_message().endpoint(MessageHandler::handle))
            .branch(Update::filter_callback_query().endpoint(CallbackHandler::handle)),
    )
    .dependencies(dptree::deps![Arc::clone(&app)])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    info!("Good Bye!");
}
