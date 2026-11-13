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
    // The displayed prefix; an unknown key is shown at its enclosing mapping's route.
    label: Option<String>,
    detail: String,
}

impl Error {
    pub(crate) fn invalid(route: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        Self::new(Category::Invalid, route, detail)
    }

    pub(crate) fn unsupported(route: impl Into<String>, detail: impl std::fmt::Display) -> Self {
        Self::new(Category::Unsupported, route, detail)
    }

    /// An error whose message is prefixed by its route, as in `jobs.build.steps: ...`.
    pub(crate) fn new(
        category: Category,
        route: impl Into<String>,
        detail: impl std::fmt::Display,
    ) -> Self {
        let route = route.into();
        Self {
            category,
            label: Some(route.clone()),
            route: Some(route),
            detail: detail.to_string(),
        }
    }

    pub(crate) fn with_category(mut self, category: Category) -> Self {
        self.category = category;
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
        if let Some(label) = &self.label {
            write!(formatter, "{label}: ")?;
        }
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for Error {}

/// A failure detected while decoding. Routes are relative until the enclosing levels prefix them.
struct Detected {
    category: Category,
    route: Option<String>,
    label: Option<String>,
    // `None` when serde_yaml or the YAML parser raised the error itself. Its message, path and
    // location are then kept verbatim and no route is attached.
    detail: Option<String>,
}

// serde errors cannot carry a typed payload, so a failure detected inside a deserializer records
// its classification here before returning the serde error, which keeps the parser's location.
thread_local! {
    static DETECTED: RefCell<Option<Detected>> = const { RefCell::new(None) };
}

fn detect(route: Option<String>, detail: Option<String>) {
    DETECTED.set(Some(Detected {
        category: Category::Invalid,
        label: route.clone(),
        route,
        detail,
    }));
}

fn detected() -> bool {
    DETECTED.with_borrow(Option::is_some)
}

/// Mark an error raised by serde_yaml or the parser, so enclosing levels leave it unrouted.
fn opaque() {
    if !detected() {
        detect(None, None);
    }
}

/// Record a typed failure detected during decoding and return its serde counterpart.
pub(crate) fn raise<E: de::Error>(error: Error) -> E {
    let message = error.to_string();
    DETECTED.set(Some(Detected {
        category: error.category,
        route: error.route,
        label: error.label,
        detail: Some(error.detail),
    }));
    E::custom(message)
}

/// Record an invalid value whose field route the enclosing mapping level supplies.
pub(crate) fn raise_unrouted<E: de::Error>(detail: &str) -> E {
    detect(None, Some(detail.to_owned()));
    E::custom(detail)
}

/// Prefix the recorded route with the enclosing location as the error leaves a nested value.
pub(crate) fn within(prefix: &str) {
    DETECTED.with_borrow_mut(|detected| {
        if let Some(
            detected @ Detected {
                detail: Some(_), ..
            },
        ) = detected
        {
            for part in [&mut detected.route, &mut detected.label] {
                *part = Some(match part.take() {
                    Some(route) => format!("{prefix}.{route}"),
                    None => prefix.to_owned(),
                });
            }
        }
    });
}

pub(crate) fn decode<T: de::DeserializeOwned>(raw: &[u8]) -> Result<T, Error> {
    DETECTED.set(None);
    serde_yaml::from_slice(raw).map_err(|error| match DETECTED.take() {
        Some(Detected {
            category,
            route,
            label,
            detail: Some(detail),
        }) => Error {
            category,
            route,
            label,
            detail: match error.location() {
                Some(at) if at.line() != 1 || at.column() != 1 => {
                    format!("{detail} at line {} column {}", at.line(), at.column())
                }
                _ => detail,
            },
        },
        _ => Error {
            category: Category::Invalid,
            route: None,
            label: None,
            detail: error.to_string(),
        },
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
                    None,
                    PhantomData,
                )))
                .inspect_err(|error| {
                    // Field and key failures are recorded below, so this is the derived struct's
                    // own unmarked error, such as a missing field.
                    if !detected() {
                        detect(None, Some(error.to_string()));
                    }
                })
            }
        }
        deserializer.deserialize_map(Mapping(PhantomData)).map(Self)
    }
}

/// The map being decoded and the most recent key, which routes failures in its value.
struct Keys<M, T>(M, Option<String>, PhantomData<T>);

impl<'de, M: MapAccess<'de>, T: Level> MapAccess<'de> for Keys<M, T> {
    type Error = M::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, M::Error> {
        self.1 = None;
        self.0
            .next_key_seed(Key::<K, T>(seed, &mut self.1, PhantomData))
            .inspect_err(|_| opaque())
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, M::Error> {
        let key = &self.1;
        self.0.next_value_seed(seed).inspect_err(|_| {
            opaque();
            // A value-level detail without a route is the failure of this field itself.
            DETECTED.with_borrow_mut(|detected| {
                if let Some(Detected {
                    route: route @ None,
                    label,
                    detail: Some(_),
                    ..
                }) = detected
                {
                    *route = key.clone();
                    *label = key.clone();
                }
            })
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct Key<'k, K, T>(K, &'k mut Option<String>, PhantomData<T>);

impl<'de, K: DeserializeSeed<'de>, T: Level> DeserializeSeed<'de> for Key<'_, K, T> {
    type Value = K::Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<K::Value, D::Error> {
        self.0
            .deserialize(KeyDeserializer::<D, T>(deserializer, self.1, PhantomData))
    }
}

struct KeyDeserializer<'k, D, T>(D, &'k mut Option<String>, PhantomData<T>);

impl<'de, D: Deserializer<'de>, T: Level> Deserializer<'de> for KeyDeserializer<'_, D, T> {
    type Error = D::Error;

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_identifier(KeyVisitor::<V, T>(visitor, self.1, PhantomData))
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_any(KeyVisitor::<V, T>(visitor, self.1, PhantomData))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum ignored_any
    }
}

struct KeyVisitor<'k, V, T>(V, &'k mut Option<String>, PhantomData<T>);

impl<V, T: Level> KeyVisitor<'_, V, T> {
    fn classify<R, E: de::Error>(key: &str, result: Result<R, E>) -> Result<R, E> {
        if let Err(error) = &result {
            let category = if T::SCHEMA.contains(&key) {
                Category::Unsupported
            } else {
                Category::Invalid
            };
            // The detail names the key, so the message shows the enclosing mapping's route.
            DETECTED.set(Some(Detected {
                category,
                route: Some(key.to_owned()),
                label: None,
                detail: Some(error.to_string()),
            }));
        }
        result
    }
}

impl<'de, V: Visitor<'de>, T: Level> Visitor<'de> for KeyVisitor<'_, V, T> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<V::Value, E> {
        *self.1 = Some(key.to_owned());
        Self::classify(key, self.0.visit_str(key))
    }

    fn visit_borrowed_str<E: de::Error>(self, key: &'de str) -> Result<V::Value, E> {
        *self.1 = Some(key.to_owned());
        Self::classify(key, self.0.visit_borrowed_str(key))
    }

    fn visit_string<E: de::Error>(self, key: String) -> Result<V::Value, E> {
        *self.1 = Some(key.clone());
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
