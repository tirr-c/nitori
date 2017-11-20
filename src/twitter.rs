use twitter_stream::TwitterStreamBuilder;
use twitter_stream::types::With;
use twitter_stream::message;
use futures::prelude::*;
use tokio_core::reactor::Handle;
use tokio_timer::{self, Timer};
use std::time::Duration;

use super::kaizo::Kaizo;
use super::error::{Error, ErrorKind};

lazy_static! {
    static ref TIMER: Timer = tokio_timer::wheel().build();
    static ref LONG_TIMER: Timer = tokio_timer::wheel().tick_duration(Duration::from_secs(1)).build();
}

#[derive(Clone, Debug)]
pub struct TweetSpec {
    pub in_reply_to: Option<u64>,
    pub text: String,
}

pub type TweetHandle = ::futures::sync::mpsc::UnboundedSender<TweetSpec>;

#[derive(Clone, Debug)]
pub struct Twitter {
    consumer_key: String,
    consumer_secret: String,
    access_key: String,
    access_secret: String,
}

impl Twitter {
    pub fn new<CK, CS, AK, AS>(consumer_key: CK, consumer_secret: CS,
                               access_key: AK, access_secret: AS) -> Self
        where CK: Into<String>, CS: Into<String>,
              AK: Into<String>, AS: Into<String>
    {
        Twitter {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            access_key: access_key.into(),
            access_secret: access_secret.into(),
        }
    }

    fn get_twitter_stream_token(&self) -> ::twitter_stream::Token<'static> {
        ::twitter_stream::Token::new(
            self.consumer_key.clone(),
            self.consumer_secret.clone(),
            self.access_key.clone(),
            self.access_secret.clone()
        )
    }
    fn get_egg_mode_token(&self) -> ::egg_mode::Token {
        use egg_mode::KeyPair;
        ::egg_mode::Token::Access {
            consumer: KeyPair::new(self.consumer_key.clone(), self.consumer_secret.clone()),
            access: KeyPair::new(self.access_key.clone(), self.access_secret.clone()),
        }
    }

    pub fn tweet(&self, handle: &Handle) -> (impl Future<Item=(), Error=Error> + 'static, TweetHandle) {
        let tweet_handle = handle.clone();
        let (tx, rx) = ::futures::sync::mpsc::unbounded();
        let (retry_tx, retry_rx) = ::futures::sync::mpsc::unbounded();
        let egg_mode_token = self.get_egg_mode_token();

        let tweet_runner = rx
            .map(|tweet: TweetSpec| {
                let mut draft = ::egg_mode::tweet::DraftTweet::new(tweet.text);
                draft.in_reply_to = tweet.in_reply_to;
                draft
            })
            .select(retry_rx)
            .map_err(|_| ErrorKind::Channel.into())
            .for_each(move |draft| {
                let retry_tx = retry_tx.clone();
                draft
                    .send(&egg_mode_token, &tweet_handle)
                    .map(|_| ())
                    .or_else(move |err| {
                        ::futures::future::result(
                            match err {
                                ::egg_mode::error::Error::TwitterError(err) => {
                                    if err.errors.iter().any(|err| err.code == 185) {
                                        retry_tx
                                            .unbounded_send(draft)
                                            .map_err(|_| ErrorKind::Channel.into())
                                    } else {
                                        Err(::egg_mode::error::Error::TwitterError(err).into())
                                    }
                                },
                                err => Err(err.into()),
                            }.into()
                        ).and_then(|_| {
                            LONG_TIMER.sleep(Duration::from_secs(300)).map_err(Into::into)
                        })
                    })
                    .and_then(|_| {
                        TIMER.sleep(Duration::from_secs(5)).map_err(Into::into)
                    })
            });
        (tweet_runner, tx)
    }
    pub fn kaizo_stream(&self,
                        handle: &Handle) -> impl Stream<Item=Kaizo, Error=Error> + 'static
    {
        let stream_handle = handle.clone();
        let stream_token = self.get_twitter_stream_token();
        let egg_mode_token = self.get_egg_mode_token();
        ::egg_mode::verify_tokens(&egg_mode_token, handle)
            .map_err(Into::into)
            .map(move |user| {
                let bot_user_id = user.id;
                TwitterStreamBuilder::user(&stream_token)
                    .handle(&stream_handle)
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
                                        let (until, _) = mentions[0].indices;
                                        let command = tweet.text.chars().take(until as usize).collect::<String>();
                                        let command = command.trim();
                                        Some(Ok(Kaizo::new(tweet.user.screen_name.clone(), tweet.id, command)))
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
}
