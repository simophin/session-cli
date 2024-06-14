use rand::Rng;

pub trait RandExt<T> {
    fn rand_ref(&self) -> &T where Self : AsRef<[T]> {
        let slice = self.as_ref();
        let index = rand::thread_rng().gen_range(0..slice.len());
        &slice[index]
    }
}

impl<T, C: AsRef<[T]>> RandExt<T> for C {}