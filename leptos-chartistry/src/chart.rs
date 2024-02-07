use crate::{
    aspect_ratio::KnownAspectRatio,
    debug::DebugRect,
    inner::InnerLayout,
    layout::{EdgeLayout, Layout},
    overlay::tooltip::Tooltip,
    projection::Projection,
    series::{RenderData, UseData},
    state::{PreState, State},
    use_watched_node::{use_watched_node, UseWatchedNode},
    AspectRatio, Padding, Series, Tick,
};
use leptos::{html::Div, *};

pub const FONT_HEIGHT: f64 = 16.0;
pub const FONT_WIDTH: f64 = 10.0;

/// Builds an SVG chart. Used inside the [Leptos view macro](https://docs.rs/leptos/latest/leptos/macro.view.html).
///
/// Check the required and optional props list near the bottom for a quick overview. There is an [assorted list of examples](https://feral-dot-io.github.io/leptos-chartistry/examples) available too.
///
/// ## Layout props
///
/// The chart is built up from layout components. Each edge has a `top`, `right`, `bottom`, and `left` prop while inside the chart has the `inner` prop. These layout props follow the builder pattern where you'll create a component, configure it to your liking, and then call [IntoEdge](crate::IntoEdge) or [IntoInner](crate::IntoInner) to get an edge layout or inner layout respectively.
///
/// ```rust
/// // TODO Example of builder pattern
/// ```
///
/// When building the component you'll have access to a [`RwSignal`](https://docs.rs/leptos/latest/leptos/struct.RwSignal.html) for each configuration option which enables fine-grained reactivity.
///
/// ```rust
/// // TODO Example of using fine-grained reactivity
/// ```
///
/// There is a shortcut where a single component may be converted to a `vec![component.into_edge()]`. This is intended to make it easier to add a single component to a layout prop. For example:
///
/// ```rust
/// <Chart aspect_ratio=AspectRatio::inner_ratio(800.0, 600.0)
///     // Shorthand for a single component:
///     top=RotatedLabel::middle("Our chart title") />
/// ```
#[component]
pub fn Chart<T: 'static, X: Tick, Y: Tick>(
    /// Determines the width and height of the chart. Charts with a different aspect ratio and axis ranges are difficult to compare. You're encouraged to pick an [inner aspect ratio](AspectRatio::inner_ratio) while the closest to a "don't think about it" approach is to automatically [use the environment](AspectRatio::environment).
    ///
    /// See [AspectRatio](AspectRatio) for a detailed explanation.
    #[prop(into)]
    aspect_ratio: MaybeSignal<AspectRatio>,

    /// The height of the font used in the chart. Passed to [SVG text](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/text). Default is 16.
    #[prop(into, optional)]
    font_height: Option<MaybeSignal<f64>>,

    /// The width must be the exact width of a monospaced character in the font used. Along with font_height, it is used to calculate the dimensions of text. These dimensions are then fed into layout composition to render the chart. The default is 10.
    #[prop(into, optional)]
    font_width: Option<MaybeSignal<f64>>,

    /// Debug mode. If enabled shows lines around components and prints render info to the console. Useful for getting an idea of how the chart is rendering itself. Below is an example of how you might use it in development. Default is false.
    ///
    /// ```rust
    /// let (debug, set_debug) = create_signal(true);
    /// view! {
    ///     <p>
    ///         <label>
    ///             <input type="checkbox" input type="checkbox"
    ///                 on:input=move |ev| set_debug.set(event_target_checked(&ev)) />
    ///             " Toggle debug mode"
    ///         </label>
    ///     </p>
    ///     <Chart
    ///         aspect_ratio=AspectRatio::inner_ratio(800.0, 600.0)
    ///         series=series
    ///         data=data
    ///         // Toggle debug on the fly
    ///         debug=debug />
    /// }
    /// ```
    #[prop(into, optional)]
    debug: MaybeSignal<bool>,

    /// Padding adds spacing around chart components. Default is the font width.
    #[prop(into, optional)]
    padding: Option<MaybeSignal<Padding>>,

    /// Top edge components. See [IntoEdge](crate::IntoEdge) for details. Default is none.
    #[prop(into, optional)]
    top: Vec<EdgeLayout<X>>,
    /// Right edge components. See [IntoEdge](crate::IntoEdge) for details. Default is none.
    #[prop(into, optional)]
    right: Vec<EdgeLayout<Y>>,
    /// Bottom edge components. See [IntoEdge](crate::IntoEdge) for details. Default is none.
    #[prop(into, optional)]
    bottom: Vec<EdgeLayout<X>>,
    /// Left edge components. See [IntoEdge](crate::IntoEdge) for details. Default is none.
    #[prop(into, optional)]
    left: Vec<EdgeLayout<Y>>,

    /// Inner chart area components. Does not render lines -- use [Series] for that. See [IntoInner](crate::IntoInner) for details. Default is none.
    #[prop(into, optional)]
    inner: Vec<InnerLayout<X, Y>>,
    /// Tooltip to show on mouse hover. See [Tooltip](crate::Tooltip) for details. Default is hidden.
    #[prop(into, optional)]
    tooltip: Tooltip<X, Y>,

    /// Series to render. Maps `T` to lines, bars, etc. See [Series] for details.
    #[prop(into)]
    series: Series<T, X, Y>,
    /// Data to render. Must be sorted.
    #[prop(into)]
    data: Signal<Vec<T>>,
) -> impl IntoView {
    let root = create_node_ref::<Div>();
    let watch = use_watched_node(root);

    // Aspect ratio signal
    let have_dimensions = create_memo(move |_| watch.bounds.get().is_some());
    let width = create_memo(move |_| watch.bounds.get().unwrap_or_default().width());
    let height = create_memo(move |_| watch.bounds.get().unwrap_or_default().height());
    let calc = AspectRatio::known_signal(aspect_ratio, width, height);

    let debug = create_memo(move |_| debug.get());
    let font_height = create_memo(move |_| font_height.map(|f| f.get()).unwrap_or(FONT_HEIGHT));
    let font_width = create_memo(move |_| font_width.map(|f| f.get()).unwrap_or(FONT_WIDTH));
    let padding = create_memo(move |_| {
        padding
            .map(|p| p.get())
            .unwrap_or_else(move || Padding::from(font_width.get()))
    });

    // Edges are added top to bottom, left to right. Layout compoeses inside out:
    let mut top = top;
    let mut left = left;
    top.reverse();
    left.reverse();

    // Build data
    let data = UseData::new(series, data);
    let pre = PreState::new(debug.into(), font_height, font_width, padding.into(), data);

    view! {
        <div class="_chartistry" style="width: fit-content; height: fit-content; overflow: visible;">
            <div node_ref=root>
                <DebugRect label="Chart" debug=debug />
                <Show when=move || have_dimensions.get() fallback=|| view!(<p>"Loading..."</p>)>
                    <RenderChart
                        watch=watch.clone()
                        pre_state=pre.clone()
                        aspect_ratio=calc
                        top=top.as_slice()
                        right=right.as_slice()
                        bottom=bottom.as_slice()
                        left=left.as_slice()
                        inner=inner.clone()
                        tooltip=tooltip.clone()
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn RenderChart<'a, X: Tick, Y: Tick>(
    watch: UseWatchedNode,
    pre_state: PreState<X, Y>,
    aspect_ratio: Memo<KnownAspectRatio>,
    top: &'a [EdgeLayout<X>],
    right: &'a [EdgeLayout<Y>],
    bottom: &'a [EdgeLayout<X>],
    left: &'a [EdgeLayout<Y>],
    inner: Vec<InnerLayout<X, Y>>,
    tooltip: Tooltip<X, Y>,
) -> impl IntoView {
    let debug = pre_state.debug;

    // Compose edges
    let (layout, edges) = Layout::compose(top, right, bottom, left, aspect_ratio, &pre_state);

    // Finalise state
    let projection = {
        let inner = layout.inner;
        let position_range = pre_state.data.position_range;
        create_memo(move |_| Projection::new(inner.get(), position_range.get())).into()
    };
    let state = State::new(pre_state, &watch, layout, projection);

    // Render edges
    let edges = edges
        .into_iter()
        .map(|r| r.render(state.clone()))
        .collect_view();

    // Inner
    let inner = inner
        .into_iter()
        .map(|opt| opt.into_use(&state).render(state.clone()))
        .collect_view();

    let outer = state.layout.outer;
    view! {
        <svg
            width=move || format!("{}px", outer.get().width())
            height=move || format!("{}px", outer.get().height())
            viewBox=move || with!(|outer| format!("0 0 {} {}", outer.width(), outer.height()))
            style="display: block; overflow: visible;">
            <DebugRect label="RenderChart" debug=debug bounds=vec![outer.into()] />
            {inner}
            {edges}
            <RenderData state=state.clone() />
        </svg>
        <Tooltip tooltip=tooltip state=state />
    }
}