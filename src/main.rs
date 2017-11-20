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

    let (tweet_runner, tweet_handle) = twitter.tweet(&handle);
    let stream = twitter.kaizo_stream(&handle)
        .inspect(|kaizo| println!("{:?}", kaizo))
        .for_each(move |kaizo| {
            let tweet = tweet_handle.clone();
            let content = if kaizo.command.chars().count() > 50 {
                format!("@{} Too long", kaizo.screen_name)
            } else {
                format!("@{} {}", kaizo.screen_name, kaizo.command)
            };
            let spec = nitori::TweetSpec {
                in_reply_to: Some(kaizo.status_id),
                text: content,
            };
            tweet.send(spec).map(|_| ()).map_err(|_| nitori::error::ErrorKind::Channel.into())
        });

    core.run(stream.join(tweet_runner).map(|_| ())).unwrap();
}
