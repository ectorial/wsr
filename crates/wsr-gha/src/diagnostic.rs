//! Provisional inspection diagnostics. Not a stable, WIT or protocol contract.

use std::cell::RefCell;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};

/// Why a source could not be inspected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Category {
    /// Invalid for the pinned GHA schema or this inspection subset's stated rules.
    Invalid,
    /// Valid GitHub Actions source that WSR does not inspect yet.
    Unsupported,
}

/// A failed inspection. Display is the diagnostic text; hosts add file context.
#[derive(Debug)]
pub struct Error {
    category: Category,
    route: Option<String>,
    message: String,
}

impl Error {
    pub(crate) fn invalid(route: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        Self::at(Category::Invalid, route, detail)
    }

    pub(crate) fn unsupported(route: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        Self::at(Category::Unsupported, route, detail)
    }

    /// An error whose message is prefixed by its route, as in `jobs.build.steps: ...`.
    fn at(category: Category, route: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        let route = route.into();
        Self::new(category, Some(route.clone()), format!("{route}: {detail}"))
    }

    pub(crate) fn new(category: Category, route: Option<String>, message: String) -> Self {
        Self {
            category,
            route,
            message,
        }
    }

    /// Replace a label-style message prefix's route with the field it refers to.
    pub(crate) fn routed(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    pub fn category(&self) -> Category {
        self.category
    }

    /// Source route of the failing field, such as `jobs.build.if`, when known at detection.
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

// serde errors cannot carry a typed payload, so a failure detected inside a deserializer records
// its classification here before returning the serde error, which keeps the parser's location.
thread_local! {
    static DETECTED: RefCell<Option<(Category, Option<String>)>> = const { RefCell::new(None) };
}

/// Record a typed failure detected during decoding and return its serde counterpart.
pub(crate) fn raise<E: de::Error>(error: Error) -> E {
    DETECTED.set(Some((error.category, error.route)));
    E::custom(error.message)
}

/// Prefix the recorded route with the enclosing location as the error leaves a nested value.
pub(crate) fn within(prefix: &str) {
    DETECTED.with_borrow_mut(|detected| {
        if let Some((_, Some(route))) = detected {
            *route = format!("{prefix}.{route}");
        }
    });
}

pub(crate) fn decode<T: de::DeserializeOwned>(raw: &[u8]) -> Result<T, Error> {
    DETECTED.set(None);
    serde_yaml::from_slice(raw).map_err(|error| {
        let (category, route) = DETECTED.take().unwrap_or((Category::Invalid, None));
        Error::new(category, route, error.to_string())
    })
}

/// A source mapping level whose unknown keys are classified against the pinned schema.
pub(crate) trait Level: de::DeserializeOwned {
    const NAME: &'static str;
    /// Keys the pinned schema accepts at this level that this subset does not inspect.
    const SCHEMA: &'static [&'static str];
}

/// Decodes `T` unchanged while observing the key at which decoding fails.
pub(crate) struct Fields<T>(pub(crate) T);

impl<'de, T: Level> de::Deserialize<'de> for Fields<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Mapping<T>(PhantomData<T>);
        impl<'de, T: Level> Visitor<'de> for Mapping<T> {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "struct {}", T::NAME)
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
                T::deserialize(de::value::MapAccessDeserializer::new(Keys::<M, T>(
                    map,
                    PhantomData,
                )))
            }
        }
        deserializer.deserialize_map(Mapping(PhantomData)).map(Self)
    }
}

struct Keys<M, T>(M, PhantomData<T>);

impl<'de, M: MapAccess<'de>, T: Level> MapAccess<'de> for Keys<M, T> {
    type Error = M::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, M::Error> {
        self.0.next_key_seed(Key::<K, T>(seed, PhantomData))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, M::Error> {
        self.0.next_value_seed(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct Key<K, T>(K, PhantomData<T>);

impl<'de, K: DeserializeSeed<'de>, T: Level> DeserializeSeed<'de> for Key<K, T> {
    type Value = K::Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<K::Value, D::Error> {
        self.0
            .deserialize(KeyDeserializer::<D, T>(deserializer, PhantomData))
    }
}

struct KeyDeserializer<D, T>(D, PhantomData<T>);

impl<'de, D: Deserializer<'de>, T: Level> Deserializer<'de> for KeyDeserializer<D, T> {
    type Error = D::Error;

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_identifier(KeyVisitor::<V, T>(visitor, PhantomData))
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_any(KeyVisitor::<V, T>(visitor, PhantomData))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum ignored_any
    }
}

struct KeyVisitor<V, T>(V, PhantomData<T>);

impl<V, T: Level> KeyVisitor<V, T> {
    fn classify<R, E>(key: &str, result: Result<R, E>) -> Result<R, E> {
        if result.is_err() {
            let category = if T::SCHEMA.contains(&key) {
                Category::Unsupported
            } else {
                Category::Invalid
            };
            DETECTED.set(Some((category, Some(key.to_owned()))));
        }
        result
    }
}

impl<'de, V: Visitor<'de>, T: Level> Visitor<'de> for KeyVisitor<V, T> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<V::Value, E> {
        Self::classify(key, self.0.visit_str(key))
    }

    fn visit_borrowed_str<E: de::Error>(self, key: &'de str) -> Result<V::Value, E> {
        Self::classify(key, self.0.visit_borrowed_str(key))
    }

    fn visit_string<E: de::Error>(self, key: String) -> Result<V::Value, E> {
        let classified = key.clone();
        Self::classify(&classified, self.0.visit_string(key))
    }

    fn visit_u64<E: de::Error>(self, key: u64) -> Result<V::Value, E> {
        self.0.visit_u64(key)
    }

    fn visit_bytes<E: de::Error>(self, key: &[u8]) -> Result<V::Value, E> {
        self.0.visit_bytes(key)
    }
}
