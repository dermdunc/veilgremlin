//! Lossless encoding of `Namespace`/`EntityType` to and from the `mapping` table's
//! columns.
//!
//! This is deliberately **not** the keying tag from `vg_core::keying` (`namespace_tag`/
//! `type_tag_for_keying`, both private): those are one-way HMAC inputs and are not meant to
//! be parsed back. Reconstructing a `Namespace`/`EntityType` at construction time — so the
//! `Keyer`'s ordinal counters can be reseeded from persisted rows (Task T05's hard
//! requirement) — needs a *round-trippable* representation, which is what this owns. Kept in
//! one place so the encode/decode halves can't drift.
//!
//! `EntityType` is `#[non_exhaustive]`, so the decode side maps any `entity_kind` string it
//! doesn't recognise onto `Custom(kind)` rather than failing — a forward-compatible landing
//! spot if a future `vg-core` adds a fixed variant that an older `vg-vault` binary then
//! reads back from a newer DB. (Encode always writes a recognised kind for today's
//! variants, so this only bites cross-version.)

use vg_core::{EntityType, Namespace, OrgId, RepoId, SessionId};

/// The `(ns_kind, ns_id)` column pair for a namespace.
pub(crate) fn namespace_columns(ns: &Namespace) -> (&'static str, String) {
    match ns {
        Namespace::Session(SessionId(uuid)) => ("session", uuid.to_string()),
        Namespace::Repo(RepoId(id)) => ("repo", id.clone()),
        Namespace::Org(OrgId(id)) => ("org", id.clone()),
    }
}

/// Reconstructs a `Namespace` from its stored `(ns_kind, ns_id)` columns. Returns `None`
/// for an unrecognised kind or an unparseable session UUID (a corrupt row); callers treat
/// that as a row to skip during reseed rather than a fatal open error.
pub(crate) fn namespace_from_columns(kind: &str, id: &str) -> Option<Namespace> {
    match kind {
        "session" => uuid::Uuid::parse_str(id)
            .ok()
            .map(|u| Namespace::Session(SessionId(u))),
        "repo" => Some(Namespace::Repo(RepoId(id.to_string()))),
        "org" => Some(Namespace::Org(OrgId(id.to_string()))),
        _ => None,
    }
}

/// The `(entity_kind, entity_custom)` column pair for an entity type. `entity_custom` is
/// `Some` only for `Custom`, carrying the raw dictionary name so two `Custom` classes whose
/// names format identically for display are still stored — and later reseeded — distinctly.
pub(crate) fn entity_columns(ty: &EntityType) -> (&'static str, Option<String>) {
    match ty {
        EntityType::Person => ("person", None),
        EntityType::Email => ("email", None),
        EntityType::Phone => ("phone", None),
        EntityType::Address => ("address", None),
        EntityType::Postcode => ("postcode", None),
        EntityType::EmployeeId => ("employee_id", None),
        EntityType::CustomerId => ("customer_id", None),
        EntityType::AccountId => ("account_id", None),
        EntityType::Iban => ("iban", None),
        EntityType::SortCode => ("sort_code", None),
        EntityType::InternalIp => ("internal_ip", None),
        EntityType::Hostname => ("hostname", None),
        EntityType::ApiKey => ("api_key", None),
        EntityType::TraceId => ("trace_id", None),
        EntityType::Password => ("password", None),
        EntityType::PrivateKey => ("private_key", None),
        EntityType::Secret => ("secret", None),
        EntityType::AccessToken => ("access_token", None),
        EntityType::Custom(name) => ("custom", Some(name.clone())),
        // `EntityType` is #[non_exhaustive]; a future fixed variant this binary predates
        // still round-trips through the `custom` column so it isn't silently merged with a
        // real `Custom` class of some other name.
        other => ("custom", Some(format!("__fixed__{other:?}"))),
    }
}

/// Reconstructs an `EntityType` from its stored `(entity_kind, entity_custom)` columns.
pub(crate) fn entity_from_columns(kind: &str, custom: Option<&str>) -> EntityType {
    match kind {
        "person" => EntityType::Person,
        "email" => EntityType::Email,
        "phone" => EntityType::Phone,
        "address" => EntityType::Address,
        "postcode" => EntityType::Postcode,
        "employee_id" => EntityType::EmployeeId,
        "customer_id" => EntityType::CustomerId,
        "account_id" => EntityType::AccountId,
        "iban" => EntityType::Iban,
        "sort_code" => EntityType::SortCode,
        "internal_ip" => EntityType::InternalIp,
        "hostname" => EntityType::Hostname,
        "api_key" => EntityType::ApiKey,
        "trace_id" => EntityType::TraceId,
        "password" => EntityType::Password,
        "private_key" => EntityType::PrivateKey,
        "secret" => EntityType::Secret,
        "access_token" => EntityType::AccessToken,
        // "custom" and any unrecognised kind land here.
        _ => EntityType::Custom(custom.unwrap_or_default().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip_ns(ns: &Namespace) {
        let (kind, id) = namespace_columns(ns);
        assert_eq!(namespace_from_columns(kind, &id).as_ref(), Some(ns));
    }

    #[test]
    fn namespace_round_trips_for_every_kind() {
        round_trip_ns(&Namespace::Session(SessionId(uuid::Uuid::from_u128(42))));
        round_trip_ns(&Namespace::Repo(RepoId("acme/widgets".to_string())));
        round_trip_ns(&Namespace::Org(OrgId("acme".to_string())));
    }

    fn round_trip_ty(ty: &EntityType) {
        let (kind, custom) = entity_columns(ty);
        assert_eq!(&entity_from_columns(kind, custom.as_deref()), ty);
    }

    #[test]
    fn entity_type_round_trips_for_fixed_and_custom_variants() {
        round_trip_ty(&EntityType::Email);
        round_trip_ty(&EntityType::AccountId);
        round_trip_ty(&EntityType::AccessToken);
        round_trip_ty(&EntityType::Custom("internal-project-codename".to_string()));
    }

    #[test]
    fn distinct_custom_names_encode_distinctly() {
        let (_, a) = entity_columns(&EntityType::Custom("foo-bar".to_string()));
        let (_, b) = entity_columns(&EntityType::Custom("foo_bar".to_string()));
        assert_ne!(a, b);
    }
}
