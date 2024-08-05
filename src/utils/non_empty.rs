use rand::Rng;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct NonEmpty<T>(Vec<T>);

impl<T> Deref for NonEmpty<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> TryFrom<Vec<T>> for NonEmpty<T> {
    type Error = ();

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(value).ok_or(())
    }
}

impl<T> NonEmpty<T> {
    pub fn head(&self) -> &T {
        self.0.get(0).unwrap()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.0.into_iter()
    }

    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    pub fn from_vec(data: Vec<T>) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        Some(Self(data))
    }

    pub fn from_iter(data: impl Iterator<Item = T>) -> Option<Self> {
        Self::from_vec(data.collect())
    }

    pub fn new(head: T, rest: impl Iterator<Item = T>) -> Self {
        let mut data = match rest.size_hint().1 {
            Some(upper) => Vec::with_capacity(upper + 1),
            _ => Vec::new(),
        };

        data.push(head);
        data.extend(rest);
        Self(data)
    }

    pub fn choose_random(&self, mut rng: impl Rng) -> &T {
        let index = rng.gen_range(0..self.len());
        &self[index]
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> NonEmpty<U> {
        NonEmpty(self.0.iter().map(f).collect())
    }
}

#[macro_export]
macro_rules! non_empty_vec {
    ($head:expr $(,$item:expr)+) => {
        crate::utils::NonEmpty::new($head, vec![$($item),+].into_iter())
    };

    ($head:expr) => {
        crate::utils::NonEmpty::new($head, std::iter::empty())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty() {
        let non_empty = non_empty_vec![1, 2, 3];
        assert_eq!(non_empty.head(), &1);

        let non_empty = NonEmpty::<u8>::from_vec(vec![]);
        assert!(non_empty.is_none());
    }
}
