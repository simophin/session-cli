pub mod identity;
mod blinded_id;

pub trait AppSetting {
    const NAME: &'static str;
}
