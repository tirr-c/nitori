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
    let twitter = nitori::Twitter::new(consumer_key, consumer_secret, access_key, access_secret);

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let stream = twitter.kaizo_stream(&handle)
        .for_each(|kaizo| {
            println!("{:?}", kaizo);
            Ok(())
        });
    let (tweet_runner, _tweet_handle) = twitter.tweet(&handle);

    core.run(stream.join(tweet_runner).map(|_| ())).unwrap();
}
