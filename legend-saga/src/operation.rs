use lapin::types::{AMQPValue, FieldTable};
use once_cell::sync::OnceCell;
use std::future::Future;

/// The AMQP header carrying the SIPLEI operation identifier.
/// The same key is used for gRPC metadata so there is a single name across transports.
pub const OPERATION_HEADER: &str = "x-operation-id";

tokio::task_local! {
    static OPERATION: Option<String>;
}

type MissingOperationHook = Box<dyn Fn(&str, &str) + Send + Sync>;

static MISSING_OPERATION_HOOK: OnceCell<MissingOperationHook> = OnceCell::new();

/// Registers the callback invoked when a consumed message carries no operation.
/// A hook keeps the library free of a metrics backend; services wire it to
/// `events_without_operation_total`. Only the first registration takes effect.
pub fn set_missing_operation_hook<F>(hook: F) -> Result<(), &'static str>
where
    F: Fn(&str, &str) + Send + Sync + 'static,
{
    MISSING_OPERATION_HOOK
        .set(Box::new(hook))
        .map_err(|_| "missing operation hook already set")
}

pub(crate) fn report_missing_operation(microservice: &str, event_type: &str) {
    if let Some(hook) = MISSING_OPERATION_HOOK.get() {
        hook(microservice, event_type);
    }
}

/// Runs `f` with `operation_id` bound to the current task.
///
/// Task-locals are not inherited by `tokio::spawn`, so anything spawned inside
/// `f` must capture the operation and re-enter this scope.
pub async fn with_operation<F, R>(operation_id: Option<String>, f: F) -> R
where
    F: Future<Output = R>,
{
    OPERATION.scope(operation_id, f).await
}

/// Returns the operation bound to the current task, if any.
pub fn current_operation() -> Option<String> {
    OPERATION.try_with(|operation| operation.clone()).ok().flatten()
}

pub(crate) fn operation_from_headers(headers: &FieldTable) -> Option<String> {
    match headers.inner().get(OPERATION_HEADER) {
        Some(AMQPValue::LongString(value)) => Some(value.to_string()),
        Some(AMQPValue::ShortString(value)) => Some(value.to_string()),
        _ => None,
    }
}

/// Adds the operation header when the current task carries one. Messages
/// published without an operation are left untouched, which keeps the
/// permissive migration window working.
pub(crate) fn apply_operation_header(headers: &mut FieldTable) {
    if let Some(operation_id) = current_operation() {
        headers.insert(
            OPERATION_HEADER.into(),
            AMQPValue::LongString(operation_id.into()),
        );
    }
}

pub(crate) fn operation_headers() -> FieldTable {
    let mut headers = FieldTable::default();
    apply_operation_header(&mut headers);
    headers
}

#[cfg(test)]
mod test_operation {
    use super::*;

    #[tokio::test]
    async fn current_operation_inside_scope() {
        let got = with_operation(Some("op-123".to_string()), async { current_operation() }).await;

        assert_eq!(got, Some("op-123".to_string()));
    }

    #[tokio::test]
    async fn current_operation_outside_scope() {
        assert_eq!(current_operation(), None);
    }

    #[tokio::test]
    async fn current_operation_with_none_bound() {
        let got = with_operation(None, async { current_operation() }).await;

        assert_eq!(got, None);
    }

    #[tokio::test]
    async fn nested_scope_overrides() {
        let got = with_operation(Some("outer".to_string()), async {
            with_operation(Some("inner".to_string()), async { current_operation() }).await
        })
        .await;

        assert_eq!(got, Some("inner".to_string()));
    }

    #[test]
    fn operation_from_headers_reads_long_string() {
        let mut headers = FieldTable::default();
        headers.insert(
            OPERATION_HEADER.into(),
            AMQPValue::LongString("op-123".into()),
        );

        assert_eq!(operation_from_headers(&headers), Some("op-123".to_string()));
    }

    #[test]
    fn operation_from_headers_missing_or_wrong_type() {
        assert_eq!(operation_from_headers(&FieldTable::default()), None);

        let mut headers = FieldTable::default();
        headers.insert(OPERATION_HEADER.into(), AMQPValue::LongInt(42));
        assert_eq!(operation_from_headers(&headers), None);
    }

    #[tokio::test]
    async fn apply_operation_header_sets_the_key() {
        let headers = with_operation(Some("op-123".to_string()), async { operation_headers() }).await;

        assert_eq!(
            operation_from_headers(&headers),
            Some("op-123".to_string())
        );
    }

    #[tokio::test]
    async fn apply_operation_header_without_operation_leaves_table_untouched() {
        let mut headers = FieldTable::default();
        headers.insert("all-micro".into(), AMQPValue::LongString("yes".into()));

        apply_operation_header(&mut headers);

        assert_eq!(operation_from_headers(&headers), None);
        assert_eq!(headers.inner().len(), 1);
    }

    #[tokio::test]
    async fn spawned_task_does_not_inherit_the_operation() {
        // Guards the documented trap: tokio::spawn drops task-locals, so audit
        // emission has to capture and re-enter the scope explicitly.
        let got = with_operation(Some("op-123".to_string()), async {
            tokio::spawn(async { current_operation() }).await.unwrap()
        })
        .await;

        assert_eq!(got, None);
    }
}
