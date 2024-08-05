pub trait MessageNamespace {
    const INT_VALUE: isize;

    const DISPLAY_NAME: &'static str;
}

macro_rules! define_namespace {
    ($name:ident, $value:literal) => {
        pub struct $name;

        impl MessageNamespace for $name {
            const INT_VALUE: isize = $value;
            const DISPLAY_NAME: &'static str = stringify!($name);
        }
    };
}

define_namespace!(DefaultNamespace, 0);
define_namespace!(ContactsNamespace, 3);
define_namespace!(GroupNamespace, 11);
define_namespace!(UserProfileConfigNamespace, 2);
define_namespace!(ConvoInfoVolatileConfigNamespace, 4);
define_namespace!(UserGroupsConfigNamespace, 5);
define_namespace!(GroupKeysNamespace, 12);
define_namespace!(GroupInfoConfigNamespace, 13);
define_namespace!(GroupMemberConfigNamespace, 14);
