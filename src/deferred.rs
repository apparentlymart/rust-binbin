/// A placeholder for a value that we'll learn only later in our process of
/// writing out data.
pub struct Deferred<'a, T> {
    _phantom: std::marker::PhantomData<&'a T>,
}
