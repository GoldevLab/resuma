use resuma::core::context::{RenderContext, RenderMode};
use resuma::core::{signal, use_computed, with_context};

#[test]
fn chained_computed_cascade_does_not_panic() {
    let ctx = RenderContext::new(RenderMode::Ssr);
    with_context(ctx, || {
        let sig = signal(1i64);

        let s1 = sig.clone();
        let c1 = use_computed(move || s1.get() * 2);

        let c1r = c1;
        let c2 = use_computed(move || c1r.get() + 1);

        sig.set(10);

        assert_eq!(c2.peek(), 21, "c2 should be sig*2+1 = 21");
    });
}

#[test]
fn conditional_computed_tracks_active_branch_only() {
    let ctx = RenderContext::new(RenderMode::Ssr);
    with_context(ctx, || {
        let cond = signal(true);
        let a = signal(1_i32);
        let b = signal(10_i32);

        let cond_r = cond.clone();
        let a_r = a.clone();
        let b_r = b.clone();
        let c = use_computed(move || if cond_r.get() { a_r.get() } else { b_r.get() });

        assert_eq!(c.peek(), 1);

        cond.set(false);
        assert_eq!(c.peek(), 10, "should follow b after cond flips");

        b.set(20);
        assert_eq!(c.peek(), 20, "should react to b in else branch");

        a.set(999);
        assert_eq!(c.peek(), 20, "should ignore a when cond is false");
    });
}
