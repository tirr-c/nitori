extern crate futures;
extern crate tokio_core;
extern crate dotenv;
extern crate nitori;

use futures::prelude::*;
use tokio_core::reactor::Core;

fn main() {
    dotenv::dotenv().ok();
    let consumer_key = dotenv::var("CONSUMER_KEY").unwrap();
    let consumer_secret = dotenv::var("CONSUMER_SECRET").unwrap();
    let access_key = dotenv::var("ACCESS_KEY").unwrap();
    let access_secret = dotenv::var("ACCESS_SECRET").unwrap();
    let token = nitori::Token {
        consumer: (consumer_key.into(), consumer_secret.into()),
        access: (access_key.into(), access_secret.into()),
    };

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let stream = nitori::kaizo_stream(token, &handle)
        .for_each(|kaizo| {
            println!("{:?}", kaizo);
            Ok(())
        });

    core.run(stream).unwrap();
}
