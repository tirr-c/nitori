error_chain! {
    foreign_links {
        TwitterStream(::twitter_stream::Error);
        TwitterStreamMessage(::twitter_stream::message::Error);
        EggMode(::egg_mode::error::Error);
        Timer(::tokio_timer::TimerError);
    }
    errors {
        Channel {
            description("channel error")
            display("Channel error")
        }
    }
}
