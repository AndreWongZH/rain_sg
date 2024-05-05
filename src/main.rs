use std::time::SystemTime;
use chrono::Local;
use log::{error, info, warn};
use teloxide::dispatching::HandlerExt;
use teloxide::types::InputFile;
use teloxide::{prelude::*, utils::command::BotCommands};

use rain_sg::engine::{self, Result};


#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    info!("Rain SG Program");

    let eng = match engine::Engine::init() {
        Ok(eng) => eng,
        Err(err) => {
            error!("Failed to init engine, err: {}", err);
            panic!("Failed to init engine, err {}", err);
        }
    };
    let bot = Bot::from_env();

    let bot_handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(
            dptree::endpoint(|msg: Message, bot: Bot| async move {
                bot.send_message(msg.chat.id, "Unrecognized command. Type /help for more commands").await?;
                respond(())
            })
        );

    Dispatcher::builder(bot, bot_handler)
        .dependencies(dptree::deps![eng])
        .default_handler(|upd| async move {
            warn!("unhandled bot reponse: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text("An error has occurred in the telegram bot dispatcher"))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start this bot")]
    Start,
    #[command(description = "show help")]
    Help,
    #[command(description = "get gif of rain info")]
    RainInfo,
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command, eng: engine::Engine) -> ResponseResult<()> {
    match cmd {
        Command::Start => bot.send_message(msg.chat.id, "SG Rain bot, enter /raininfo").await?,
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::RainInfo => {
            let current_time = Local::now();
            bot.send_message(msg.chat.id, format!("Current time now is: {}, please wait while the gif is being generated", current_time.to_string())).await?;

            let gif_name = match eng.generate_current_weather_condition().await {
                Ok(gif_name) => gif_name,
                Err(err) => {
                    error!("Failed to generate current weather condition, err: {}", err);
                    // skip this error
                    return Ok(());
                }
            };
            send_animation_from_file(bot, msg, &gif_name).await.unwrap()
        }
    };

    Ok(())
}


async fn send_animation_from_file(bot: Bot, msg: Message, file_name: &str) -> Result<Message> {
    let gif_file = InputFile::file(file_name);

    // Send the GIF to the user
    match bot.send_animation(msg.chat.id, gif_file).send().await {
        Ok(res) => {
            info!("GIF sent successfully!");
           Ok(res) 
        },
        Err(err) => {
            error!("Failed to send GIF: {}", err);
            Err(err.into())
        },
    }
}

fn setup_logger() -> core::result::Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("program.log")?)
        .apply()?;
    Ok(())
}