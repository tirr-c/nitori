use twitter_stream::TwitterStreamBuilder;
use twitter_stream::types::With;
use twitter_stream::message;
use futures::prelude::*;
use tokio_core::reactor::Handle;

use super::kaizo::Kaizo;
use super::error::Error;

use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Token {
    pub consumer: (Cow<'static, str>, Cow<'static, str>),
    pub access: (Cow<'static, str>, Cow<'static, str>),
}

pub fn kaizo_stream<'a>(token: Token,
                        handle: &'a Handle) -> impl Stream<Item=Kaizo, Error=Error> + 'a
{
    use egg_mode::KeyPair;
    let stream_token = ::twitter_stream::Token {
        consumer_key: token.consumer.0.clone(),
        consumer_secret: token.consumer.1.clone(),
        access_key: token.access.0.clone(),
        access_secret: token.access.1.clone(),
    };
    let egg_mode_token = ::egg_mode::Token::Access {
        consumer: KeyPair::new(token.consumer.0, token.consumer.1),
        access: KeyPair::new(token.access.0, token.access.1),
    };

    ::egg_mode::verify_tokens(&egg_mode_token, handle)
        .map_err(Into::into)
        .map(move |user| {
            let bot_user_id = user.id;
            TwitterStreamBuilder::user(&stream_token)
                .handle(handle)
                .with(Some(With::User))
                .replies(true)
                .listen()
                .flatten_stream()
                .map_err(Into::into)
                .filter_map(move |msg_json| {
                    let message = match message::from_str(&msg_json) {
                        Ok(x) => x,
                        Err(x) => return Some(Err(x.into())),
                    };
                    match message {
                        message::StreamMessage::Tweet(tweet) => {
                            if tweet.retweeted_status.is_some() { None }
                            else if tweet.quoted_status.is_some() { None }
                            else {
                                let mentions = &tweet.entities.user_mentions;
                                if mentions.len() != 1 { None }
                                else if mentions[0].id != bot_user_id { None }
                                else {
                                    Some(Ok(Kaizo))
                                }
                            }
                        },
                        _ => None,
                    }
                })
                .and_then(|x| x)
        })
        .flatten_stream()
}
