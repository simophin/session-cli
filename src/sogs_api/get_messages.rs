use std::borrow::Cow;

use crate::http_api::HttpJsonApi;
use crate::utils::NonEmptyStringRef;
use http::Method;
use serde::Serialize;

pub struct GetRecentMessages<'a> {
    pub room: NonEmptyStringRef<'a>,
    pub limit: Option<usize>,
}

pub struct GetMessagesBefore<'a> {
    pub room: NonEmptyStringRef<'a>,
    pub before_msg_id: NonEmptyStringRef<'a>,
    pub limit: Option<usize>,
}

pub struct GetMessagesSince<'a> {
    pub room: NonEmptyStringRef<'a>,
    pub since_msg_id: NonEmptyStringRef<'a>,
    pub limit: Option<usize>,
}

fn build_get_message_path_segments<'a>(
    room: &'a str,
    operation: &'a str,
) -> impl Iterator<Item = Cow<'a, str>> {
    return ["room", room, "messages", operation]
        .into_iter()
        .map(Cow::Borrowed);
}

fn build_limit_query<'a>(
    limit: Option<usize>,
) -> impl Iterator<Item = (Cow<'a, str>, Cow<'a, str>)> {
    return limit
        .into_iter()
        .flat_map(|limit| [(Cow::Borrowed("limit"), Cow::Owned(limit.to_string()))].into_iter());
}

impl<'a> HttpJsonApi for GetRecentMessages<'a> {
    type SuccessResponse = Vec<super::message::Message<'static>>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path_segments(&self) -> impl Iterator<Item = Cow<str>> {
        build_get_message_path_segments(self.room.as_str(), "recent")
    }

    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        build_limit_query(self.limit)
    }

    fn request(&self) -> Option<&impl Serialize> {
        Option::<&()>::None
    }
}

impl<'a> HttpJsonApi for GetMessagesBefore<'a> {
    type SuccessResponse = Vec<super::message::Message<'static>>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path_segments(&self) -> impl Iterator<Item = Cow<str>> {
        build_get_message_path_segments(self.room.as_str(), "before")
            .chain(std::iter::once(Cow::Borrowed(self.before_msg_id.as_str())))
    }

    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> + '_ {
        build_limit_query(self.limit)
    }

    fn request(&self) -> Option<&impl Serialize> {
        Option::<&()>::None
    }
}

impl<'a> HttpJsonApi for GetMessagesSince<'a> {
    type SuccessResponse = Vec<super::message::Message<'static>>;

    fn method(&self) -> Method {
        Method::GET
    }

    fn path_segments(&self) -> impl Iterator<Item = Cow<str>> {
        build_get_message_path_segments(self.room.as_str(), "since")
            .chain(std::iter::once(Cow::Borrowed(self.since_msg_id.as_str())))
    }

    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> + '_ {
        build_limit_query(self.limit)
    }

    fn request(&self) -> Option<&impl Serialize> {
        Option::<&()>::None
    }
}
