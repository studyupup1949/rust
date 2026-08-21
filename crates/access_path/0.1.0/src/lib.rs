#![cfg_attr(not(test), no_std)]

/// # KeyPath — Type-safe field paths for Rust structs
///
/// A `KeyPath` represents a path from a root type to a field value,
/// enabling type-safe field access without repeating boilerplate.
///
/// ## Basic usage
///
/// ```rust
/// use access_path::KeyPath;
///
/// #[derive(KeyPath)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let p = Person { name: "Alice".into(), age: 30 };
/// let name: &String = Person::name_path().get(&p);
/// assert_eq!(name, "Alice");
///
/// let mut p = Person { name: "Bob".into(), age: 25 };
/// *Person::name_path().get_mut(&mut p) = "Charlie".into();
/// assert_eq!(p.name, "Charlie");
///
/// Person::age_path().set(&mut p, 35);
/// assert_eq!(p.age, 35);
///
/// // Compose paths for nested structs:
/// #[derive(KeyPath)]
/// struct Company {
///     ceo: Person,
///     name: String,
/// }
///
/// let ceo_age = Company::ceo_path().then(Person::age_path());
/// let c = Company { ceo: Person { name: "A".into(), age: 42 }, name: "X".into() };
/// assert_eq!(*ceo_age.get(&c), 42);
/// ```
pub trait KeyPath<Root: ?Sized> {
    /// The field type this path accesses.
    type Value: ?Sized;

    /// Access the field immutably.
    fn get<'a>(&self, root: &'a Root) -> &'a Self::Value
    where
        Self::Value: 'a;

    /// Access the field mutably.
    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Self::Value
    where
        Self::Value: 'a;

    /// Set the field to a new value.
    fn set(&self, root: &mut Root, value: Self::Value)
    where
        Self::Value: Sized,
    {
        *self.get_mut(root) = value;
    }

    /// Compose with another `KeyPath` to access a sub-field.
    ///
    /// Returns a [`Composed`] path with inherent `get`/`get_mut`/`set` methods.
    fn then<Sub>(self, sub: Sub) -> Composed<Self, Sub>
    where
        Self: Sized,
        Self::Value: Sized,
        Sub: KeyPath<Self::Value>,
    {
        Composed(self, sub)
    }
}

// ── Composed path ───────────────────────────────────────────────

/// A composed path created by [`KeyPath::then`].
///
/// Provides inherent `get`/`get_mut`/`set` methods (does not implement `KeyPath`).
pub struct Composed<A, B>(pub A, pub B);

impl<A, B> Composed<A, B> {
    /// Access the nested field immutably.
    pub fn get<'a, Root>(&self, root: &'a Root) -> &'a B::Value
    where
        A: KeyPath<Root>,
        A::Value: 'a,
        B: KeyPath<A::Value>,
    {
        self.1.get(self.0.get(root))
    }

    /// Access the nested field mutably.
    pub fn get_mut<'a, Root>(&self, root: &'a mut Root) -> &'a mut B::Value
    where
        A: KeyPath<Root>,
        A::Value: 'a,
        B: KeyPath<A::Value>,
    {
        self.1.get_mut(self.0.get_mut(root))
    }

    /// Set the nested field to a new value.
    pub fn set<'a, Root>(&self, root: &'a mut Root, value: B::Value)
    where
        A: KeyPath<Root>,
        A::Value: 'a,
        B: KeyPath<A::Value>,
        B::Value: Sized,
    {
        let mid = self.0.get_mut(root);
        self.1.set(mid, value);
    }
}

pub use access_path_derive::KeyPath;
