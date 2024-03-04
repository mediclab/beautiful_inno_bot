#[macro_use]
extern crate log;
extern crate inflector;

use callback::CallbackHandler;
use command::{BotCommand, CommandHandler};
use dotenv::dotenv;
use envconfig::Envconfig;
use message::MessageHandler;
use std::sync::Arc;
use teloxide::{adaptors::DefaultParseMode, prelude::*, types::ParseMode};

mod bot;
mod callback;
mod command;
mod exif;
mod image;
mod message;

#[derive(Clone)]
pub struct Application {
    bot: DefaultParseMode<Bot>,
    config: Config,
}

#[derive(Envconfig, Clone)]
pub struct Config {
    #[envconfig(from = "ADMIN_USER_ID")]
    pub admin: i64,
    #[envconfig(from = "GROUP_ID")]
    pub group_id: i64,
    #[envconfig(from = "CARGO_PKG_VERSION", default = "unknown")]
    pub version: String,
}

impl Application {
    pub fn new() -> Self {
        Self {
            bot: Bot::from_env().parse_mode(ParseMode::Html),
            config: Config::init_from_env().expect("Can't load config"),
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

    let _guard = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    });

    let app = Arc::new(Application::new());

    info!("Bot version: {}", &app.config.version);

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
