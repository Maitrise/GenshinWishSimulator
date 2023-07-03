#![allow(dead_code)]

use teloxide::{prelude::*, utils::command::BotCommands, types::ParseMode};
use std::sync::Arc;
mod rolls;
use rolls::{Gacha, Item, ItemType};
use mongodb::{Client, options::{ClientOptions, UpdateOptions}, Collection, bson::{doc, Document}};
#[macro_use]
extern crate lazy_static;


lazy_static! {
    static ref ITEMS: Vec<Item> = Gacha::from_json("src/items.json");
}


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Wish")]
    Wish,
}

async fn multi_roll(bot: Bot, msg: Message, client: Arc<Client>) -> ResponseResult<()> {

    let db = client.database("gacha_db");
    let user_gacha_data: Collection<Document> = db.collection("user_gacha_data");

    let filter = doc! { "user_id": msg.from().expect("No user").id.to_string() };
    let user_doc_option = user_gacha_data.find_one(filter.clone(), None).await.unwrap();
    let (five_star_pity_chances, four_star_pity_chances) = if let Some(user_doc) = user_doc_option {
        (user_doc.get_i32("five_star_pity_chances").unwrap_or(0), user_doc.get_i32("four_star_pity_chances").unwrap_or(0))
    } else {
        (0, 0)
    };
    let mut gacha = Gacha {
        items: ITEMS.clone(),
        five_star_pity_counter: five_star_pity_chances,
        four_star_pity_counter: four_star_pity_chances,
        is_last_five_star_item_limited: false,
    };

    let result = gacha.roll();
    
    let update = doc! { "$set": { "five_star_pity_chances": gacha.five_star_pity_counter, "four_star_pity_chances": gacha.four_star_pity_counter } };
    user_gacha_data.update_one(filter, update, UpdateOptions::builder().upsert(true).build()).await.unwrap();
    let sender = &msg.from().unwrap().first_name;
    let stars: &str = match result.item_type {
        ItemType::FiveStarCharacterLimited => "A limited five star character",
        ItemType::FiveStarCharacterStandard => "A standard five star character",
        ItemType::FiveStarWeapon => "A five star weapon",
        ItemType::FourStarWeapon => "A four star weapon",
        ItemType::FourStarCharacter => "A four star character",
        ItemType::ThreeStarItem => "A three star weapon"
    };

    let reply: String = format!("{}\n\n{}, You have received {}!<a href='{}'>⠀</a>\n\n{} is a {}", result.description, sender, result.name, result.image_url, result.name, stars);
    

    let _ = bot.send_message(msg.chat.id, reply)
    .parse_mode(ParseMode::Html).
    await;
    Ok(())
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    client: Arc<Client>,
    _me: teloxide::types::Me
) -> ResponseResult<()> {
    match cmd {
        Command::Wish => {
            multi_roll(bot, msg, client).await
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot = Bot::from_env();
    let bot_clone = bot.clone();

    let client_options = ClientOptions::parse("XYZ").await.unwrap();
    let client: Arc<Client> = Arc::new(Client::with_options(client_options).unwrap());
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer),
        );

    Dispatcher::builder(bot_clone, handler)
        .dependencies(dptree::deps![client, bot.clone()])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
