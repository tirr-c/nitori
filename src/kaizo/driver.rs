use futures::prelude::*;

use error::{Error, ErrorKind};
use timer::LONG_TIMER;
use twitter::{TweetHandle, TweetSpec};
use kaizo::Kaizo;

use std::time::Duration;

pub type KaizoHandle = ::futures::sync::mpsc::UnboundedSender<Kaizo>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FailureKind {
    Normal,
    Explosion,
    Heroes,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum KaizoResult {
    Success,
    Failure(FailureKind),
}

#[derive(Clone, Debug)]
struct KaizoPlan {
    kaizo: Kaizo,
    expected_time: u64,
    actual_time: u64,
    result: KaizoResult,
}

pub fn drive(tweet_handle: TweetHandle)
    -> (impl Future<Item=(), Error=Error> + 'static, KaizoHandle)
{
    use ::std::hash::{Hash, Hasher};

    let (tx, rx) = ::futures::sync::mpsc::unbounded();
    let driver = rx
        .map_err(|_| ErrorKind::Channel.into())
        .map(|kaizo: Kaizo| {
            let mut hasher = ::twox_hash::XxHash::with_seed(kaizo.status_id);
            kaizo.hash(&mut hasher);
            let hash = hasher.finish();
            let chance = (hash & 0xff) as u8;
            let expected_time = ((hash & 0xff00) >> 8) + 180;
            let actual_time = (((hash & 0xff0000) >> 16) % (expected_time - 60)) + 60;
            let kaizo_result = if chance < 0x90 {
                // succeeds
                KaizoResult::Success
            } else if chance < 0xd0 {
                // fails normally
                KaizoResult::Failure(FailureKind::Normal)
            } else if chance < 0xf0 {
                // fails with explosion
                KaizoResult::Failure(FailureKind::Explosion)
            } else {
                // fails with Heroes of the Storm
                KaizoResult::Failure(FailureKind::Heroes)
            };
            KaizoPlan {
                kaizo,
                expected_time,
                actual_time,
                result: kaizo_result,
            }
        })
        .for_each(move |plan| {
            let tweet_handle = tweet_handle.clone().sink_map_err(
                |_| ErrorKind::Channel.into()
            );
            let expected_time_min = (plan.expected_time / 60) +
                if plan.expected_time % 60 >= 30 { 1 } else { 0 };
            let actual_time = plan.actual_time;

            let tweet = format!("@{} '{}' -> '{}'의 개조를 시작했어. {}분 정도 걸릴 거야.",
                                plan.kaizo.screen_name,
                                plan.kaizo.from, plan.kaizo.to,
                                expected_time_min);
            let spec = TweetSpec {
                in_reply_to: Some(plan.kaizo.status_id),
                text: tweet,
            };
            tweet_handle
                .send(spec)
                .and_then(move |tweet_handle| {
                    LONG_TIMER
                        .sleep(Duration::from_secs(actual_time))
                        .map_err(Into::into)
                        .map(move |_| tweet_handle)
                })
                .and_then(move |tweet_handle| {
                    let tweet = match plan.result {
                        KaizoResult::Success => {
                            format!("@{} '{}' -> '{}'의 개조에 성공했어.",
                                    plan.kaizo.screen_name,
                                    plan.kaizo.from, plan.kaizo.to)
                        },
                        KaizoResult::Failure(FailureKind::Normal) => {
                            format!("@{} '{}' -> '{}'의 개조에 실패했어.",
                                    plan.kaizo.screen_name,
                                    plan.kaizo.from, plan.kaizo.to)
                        },
                        KaizoResult::Failure(FailureKind::Explosion) => {
                            format!("@{} '{}'을 개조하다가 폭발해 버렸어.",
                                    plan.kaizo.screen_name,
                                    plan.kaizo.from)
                        },
                        KaizoResult::Failure(FailureKind::Heroes) => {
                            format!("@{} '{}'가 시공의 폭풍에 휩쓸려 버렸어.",
                                    plan.kaizo.screen_name,
                                    plan.kaizo.from)
                        },
                    };
                    let spec = TweetSpec {
                        in_reply_to: Some(plan.kaizo.status_id),
                        text: tweet,
                    };
                    tweet_handle.send(spec).map(|_| ())
                })
        });
    (driver, tx)
}
