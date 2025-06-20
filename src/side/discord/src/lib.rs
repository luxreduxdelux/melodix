use std::time::UNIX_EPOCH;

use discord_presence::{Client, Event};
use mlua::prelude::*;

struct Discord(Client);

impl mlua::UserData for Discord {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "set_state",
            |_, this, (text_a, text_b, image, time): (String, String, String, u32)| {
                // Set the activity
                let time_a = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let time_b = time_a + time as u64;

                this.0
                    .set_activity(|act| {
                        act.details(text_a)
                            .assets(|f| f.large_image(image))
                            .state(text_b)
                            ._type(discord_presence::models::ActivityType::Listening)
                            .timestamps(|f| f.start(time_a).end(time_b))
                    })
                    .expect("Failed to set activity");
                Ok(())
            },
        );
    }
}

#[mlua::lua_module]
fn melodix_discord(_: &Lua) -> LuaResult<Discord> {
    // Create the client
    let mut client = Client::new(1385408557796687923);

    // Start up the client connection, so that we can actually send and receive stuff
    client.start();

    client.block_until_event(Event::Ready).unwrap();

    Ok(Discord(client))
}
