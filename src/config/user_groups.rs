use super::ConfigExt;
use crate::bindings;
use crate::clock::Timestamp;
use crate::cwrapper::{CWrapper, OwnedCWrapper};
use crate::ed25519::ED25519SecKey;
use crate::session_id::GroupID;
use crate::utils::CArrayExt;
use anyhow::bail;
use derive_more::{Deref, DerefMut};
use std::ffi::{c_char, CStr};
use std::mem::MaybeUninit;

struct GroupIter(CWrapper<bindings::user_groups_iterator>);

pub struct LegacyGroupInfo(OwnedCWrapper<bindings::ugroups_legacy_group_info>);

#[derive(Deref, DerefMut, Eq, PartialEq, Clone)]
pub struct GroupInfo(bindings::ugroups_group_info);

#[derive(Deref, DerefMut)]
pub struct CommunityInfo(bindings::ugroups_community_info);

pub enum Group {
    Legacy(LegacyGroupInfo),
    Group(GroupInfo),
    Community(CommunityInfo),
}

pub type GroupAuthData = [u8; 100];

impl GroupInfo {
    pub fn group_id(&self) -> Option<GroupID> {
        let id = unsafe { CStr::from_ptr(self.0.id.as_ptr()) }
            .to_str()
            .ok()?;
        id.parse().ok()
    }

    pub fn set_group_id(&mut self, group_id: &GroupID) {
        self.0.id.as_mut_slice().copy_from_slice(unsafe {
            std::mem::transmute(group_id.as_c_str().to_bytes_with_nul())
        });
    }

    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.0.name.as_ptr()) }
            .to_str()
            .unwrap()
    }

    pub fn set_name(&mut self, name: &str) {
        let len = name.len().min(self.0.name.len() - 1);
        (&mut self.0.name.as_mut_slice()[..len]).copy_from_slice(
            &unsafe { std::mem::transmute::<_, &[c_char]>(name.as_bytes()) }[..len],
        );
        self.0.name[len] = 0;
    }

    pub fn sec_key(&self) -> Option<ED25519SecKey> {
        if self.0.have_secretkey {
            Some(self.0.secretkey.into())
        } else {
            None
        }
    }

    pub fn clear_sec_key(&mut self) {
        self.0.secretkey.fill(0);
        self.0.have_secretkey = false;
    }

    pub fn auth_data(&self) -> Option<&GroupAuthData> {
        if self.have_auth_data {
            Some(&self.auth_data)
        } else {
            None
        }
    }

    pub fn clear_auth_data(&mut self) {
        self.auth_data.fill(0);
        self.have_auth_data = false;
    }

    pub fn joined_at(&self) -> Option<Timestamp> {
        Timestamp::from_mills(self.joined_at)
    }

    pub fn is_kicked(&self) -> bool {
        unsafe { bindings::ugroups_group_is_kicked(&self.0) }
    }

    pub fn set_kicked(&mut self) {
        unsafe { bindings::ugroups_group_set_kicked(&mut self.0) }
    }
}

impl CommunityInfo {
    pub fn base_url(&self) -> &str {
        self.base_url.cstr_to_str().unwrap_or_default()
    }

    pub fn set_base_url(&mut self, base_url: &str) {
        let len = base_url.len().min(self.0.base_url.len() - 1);
        (&mut self.0.base_url.as_mut_slice()[..len]).copy_from_slice(
            &unsafe { std::mem::transmute::<_, &[c_char]>(base_url.as_bytes()) }[..len],
        );
        self.0.base_url[len] = 0;
    }

    pub fn url_as_key(&self) -> String {
        format!("{}/{}", self.base_url(), self.room())
    }

    pub fn room(&self) -> &str {
        self.room.cstr_to_str().unwrap_or_default()
    }

    pub fn set_room(&mut self, room: &str) -> anyhow::Result<()> {
        if !self.room.write_cstr(room) {
            bail!(
                "Room name is too long, right now only {} is supported",
                self.room.len() - 1
            );
        }

        Ok(())
    }
}

union GroupUnion {
    legacy: bindings::ugroups_legacy_group_info,
    group: bindings::ugroups_group_info,
    community: bindings::ugroups_community_info,
}

impl Iterator for GroupIter {
    type Item = Group;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.0.as_mut_ptr();
        unsafe {
            if bindings::user_groups_iterator_done(ptr) {
                return None;
            }

            let mut group_info: GroupUnion = MaybeUninit::zeroed().assume_init();

            let ret = if bindings::user_groups_it_is_legacy_group(ptr, &mut group_info.legacy) {
                Some(Group::Legacy(LegacyGroupInfo(OwnedCWrapper::new(
                    group_info.legacy,
                    bindings::ugroups_legacy_group_free,
                ))))
            } else if bindings::user_groups_it_is_group(ptr, &mut group_info.group) {
                Some(Group::Group(GroupInfo(group_info.group)))
            } else if bindings::user_groups_it_is_community(ptr, &mut group_info.community) {
                Some(Group::Community(CommunityInfo(group_info.community)))
            } else {
                None
            };

            bindings::user_groups_iterator_advance(ptr);

            ret
        }
    }
}

impl super::UserGroupsConfig {
    pub fn get_groups(&self) -> impl Iterator<Item = Group> {
        unsafe {
            let iter = bindings::user_groups_iterator_new(self.as_ref() as *const _);
            CWrapper::new_with_destroyer(iter, bindings::user_groups_iterator_free)
                .into_iter()
                .flat_map(GroupIter)
        }
    }

    pub fn get_or_create_group(
        &mut self,
        group_id: &str,
    ) -> anyhow::Result<bindings::ugroups_group_info> {
        unsafe {
            let group_id = std::ffi::CString::new(group_id).unwrap();
            let mut out: bindings::ugroups_group_info = MaybeUninit::zeroed().assume_init();
            if bindings::user_groups_get_or_construct_group(
                self.as_mut() as *mut _,
                &mut out,
                group_id.as_ptr(),
            ) {
                Ok(out)
            } else {
                anyhow::bail!(
                    "Failed to get or create group, last error = {:?}",
                    self.last_error()
                );
            }
        }
    }

    pub fn set_group(&mut self, g: &Group) -> anyhow::Result<()> {
        let ptr = self.as_mut() as *mut _;
        unsafe {
            match g {
                Group::Group(GroupInfo(info)) => bindings::user_groups_set_group(ptr, info),

                Group::Legacy(LegacyGroupInfo(info)) => {
                    bindings::user_groups_set_legacy_group(ptr, info.deref())
                }

                Group::Community(CommunityInfo(info)) => {
                    bindings::user_groups_set_community(ptr, info)
                }
            }
        };

        if let Some(err) = self.last_error() {
            anyhow::bail!("Failed to set group, last error = {:?}", err);
        } else {
            Ok(())
        }
    }

    pub fn remove_group(&mut self, group_id: &GroupID) {
        unsafe {
            bindings::user_groups_erase_group(
                self.as_mut() as *mut _,
                group_id.as_c_str().as_ptr(),
            );
        }
    }
}
