use crate::db::Repository;

mod conversation;

pub struct State<'a> {
    pub(self) repo: &'a Repository,
}
