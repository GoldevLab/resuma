//! Typed extractors, reactive For/Match.

use resuma::flow::{FromFlowRequest, Path, Query};
use resuma::prelude::*;
use std::collections::BTreeMap;

#[derive(serde::Deserialize)]
struct Search {
    q: String,
}

#[test]
fn path_extractor_single_param() {
    let mut params = BTreeMap::new();
    params.insert("id".into(), "7".into());
    let req = FlowRequest::from_parts("GET", "/x/7", BTreeMap::new(), params, BTreeMap::new());
    let Path(id): Path<u64> = Path::from_request(&req).unwrap();
    assert_eq!(id, 7);
}

#[test]
fn query_extractor_struct() {
    let mut query = BTreeMap::new();
    query.insert("q".into(), "hi".into());
    let req = FlowRequest::from_parts("GET", "/", BTreeMap::new(), BTreeMap::new(), query);
    let Query(s): Query<Search> = Query::from_request(&req).unwrap();
    assert_eq!(s.q, "hi");
}

#[test]
fn reactive_for_ssr_marker() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};
    let ctx = RenderContext::new(RenderMode::Ssr);
    let html = with_context(ctx, || {
        let rows = resuma::signal(vec![1_i32, 2]);
        let v = for_signal(&rows, None, |n| vec![Child::Text(n.to_string())]);
        render_view(&v)
    });
    assert!(html.contains("resuma-for"));
}

#[test]
fn reactive_match_ssr_marker() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};
    let ctx = RenderContext::new(RenderMode::Ssr);
    let html = with_context(ctx, || {
        let mode = resuma::signal("a".to_string());
        let v = match_signal(
            &mode,
            vec![("a".into(), vec![Child::Text("A".into())])],
            None,
        );
        render_view(&v)
    });
    assert!(html.contains("resuma-match"));
}
