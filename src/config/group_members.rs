use super::GroupMemberConfig;
use crate::bindings;
use crate::cwrapper::CIteratorWrapper;
use crate::session_id::IndividualID;
use crate::utils::CArrayExt;
use derive_more::Deref;
use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;

#[derive(Deref)]
pub struct GroupMember {
    #[deref]
    member: bindings::config_group_member,
    id: IndividualID,
}

impl GroupMember {
    pub fn new(member: bindings::config_group_member) -> Option<Self> {
        let id = IndividualID::from_c_string_array(&member.session_id)?;
        Some(Self { member, id })
    }

    pub fn name(&self) -> &str {
        self.member.name.cstr_to_str().unwrap_or_default()
    }

    pub fn session_id(&self) -> &IndividualID {
        &self.id
    }
}

impl Debug for GroupMember {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupMember")
            .field("name", &self.name())
            .field("session_id", &self.session_id())
            .finish()
    }
}

impl GroupMemberConfig {
    pub fn members(&self) -> impl Iterator<Item = GroupMember> + 'static {
        CIteratorWrapper::new(
            unsafe { bindings::groups_members_iterator_new(self.as_ref() as *const _) },
            bindings::groups_members_iterator_free,
            bindings::groups_members_iterator_done,
            bindings::groups_members_iterator_advance,
        )
        .filter_map(GroupMember::new)
    }

    pub fn set_member(&mut self, member: &GroupMember) {
        unsafe {
            bindings::groups_members_set(self.as_mut() as *mut _, &member.member);
        }
    }

    pub fn remove_member(&mut self, id: &IndividualID) -> bool {
        unsafe { bindings::groups_members_erase(self.as_mut() as *mut _, id.as_c_str().as_ptr()) }
    }

    pub fn get(&mut self, id: &IndividualID) -> Option<GroupMember> {
        unsafe {
            let mut member = MaybeUninit::zeroed().assume_init();
            if bindings::groups_members_get(
                self.as_mut() as *mut _,
                &mut member,
                id.as_c_str().as_ptr(),
            ) {
                GroupMember::new(member)
            } else {
                None
            }
        }
    }

    pub fn get_or_construct_member(&mut self, id: &IndividualID) -> Option<GroupMember> {
        unsafe {
            let mut member = MaybeUninit::zeroed().assume_init();
            if bindings::groups_members_get_or_construct(
                self.as_mut() as *mut _,
                &mut member,
                id.as_c_str().as_ptr(),
            ) {
                GroupMember::new(member)
            } else {
                None
            }
        }
    }
}

impl Debug for GroupMemberConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.members()).finish()
    }
}
