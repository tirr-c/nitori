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
    let (driver, kaizo_handle) = nitori::kaizo::drive(tweet_handle.clone());
    let stream = twitter.kaizo_stream(&handle)
        .inspect(|kaizo| println!("{:?}", kaizo))
        .for_each(move |kaizo| {
            let tweet = tweet_handle.clone().sink_map_err(
                |_| nitori::error::ErrorKind::Channel.into()
            );
            let kaizo_handle = kaizo_handle.clone().sink_map_err(
                |_| nitori::error::ErrorKind::Channel.into()
            );

            let content = if kaizo.from.chars().count() > 30 || kaizo.to.chars().count() > 30 {
                format!("@{} 물건 이름이 너무 길어.", kaizo.screen_name)
            } else {
                format!("@{} '{}' -> '{}' 말이지? 알겠어!",
                        kaizo.screen_name, kaizo.from, kaizo.to)
            };
            let spec = nitori::TweetSpec {
                in_reply_to: Some(kaizo.status_id),
                text: content,
            };
            tweet.send(spec).and_then(|_| {
                kaizo_handle.send(kaizo).map(|_| ())
            })
        });

    core.run(stream.join3(tweet_runner, driver).map(|_| ())).unwrap();
}
