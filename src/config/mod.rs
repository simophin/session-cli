use crate::bindings;

mod config_auto_impl;
mod contacts;
mod group_info;
mod group_keys;
mod group_members;
mod groups;
mod individuals;
mod user_groups;
mod user_profile;

use crate::oxen_api::retrieve::Message;
pub use group_keys::*;
pub use groups::*;
pub use individuals::*;
pub use user_groups::*;

pub trait NamedConfig {
    const CONFIG_TYPE_NAME: &'static str;
}

pub trait Config: NamedConfig {
    type MergeArg<'a>;
    type PushData;

    fn config_type_name(&self) -> &str {
        <Self as NamedConfig>::CONFIG_TYPE_NAME
    }

    fn merge<'a>(
        &mut self,
        messages: &'a [Message],
        arg: Self::MergeArg<'a>,
    ) -> Result<usize, String>;

    fn current_hashes(&self) -> Vec<String>;

    fn push(&mut self) -> anyhow::Result<Self::PushData>;

    fn confirm_pushed(&mut self, seq: bindings::seqno_t, msg_hash: &str);

    fn needs_push(&self) -> bool;

    fn needs_dump(&self) -> bool;

    fn dump(&mut self) -> Option<impl AsRef<[u8]> + 'static>;

    fn to_json(&self) -> anyhow::Result<serde_json::Value>;
}

trait ConfigExt: Config {
    fn last_error(&self) -> Option<&str>;
}

#[macro_export]
macro_rules! define_config_type {
    ($name:ident) => {
        pub struct $name(crate::cwrapper::CWrapper<crate::bindings::config_object>);

        impl crate::config::NamedConfig for $name {
            const CONFIG_TYPE_NAME: &'static str = stringify!($name);
        }

        impl AsRef<crate::bindings::config_object> for $name {
            fn as_ref(&self) -> &crate::bindings::config_object {
                self.0.as_ref()
            }
        }

        impl AsMut<crate::bindings::config_object> for $name {
            fn as_mut(&mut self) -> &mut crate::bindings::config_object {
                self.0.as_mut()
            }
        }

        impl From<crate::cwrapper::CWrapper<crate::bindings::config_object>> for $name {
            fn from(wrapper: crate::cwrapper::CWrapper<crate::bindings::config_object>) -> Self {
                $name(wrapper)
            }
        }
    };
}
