use std::{fmt::Debug, time::Duration};

use num::FromPrimitive;
use leptos::{prelude::*, either::Either, logging::log, reactive::wrappers::write::SignalSetter};
use thaw::{
    Body1, Button, ButtonAppearance, Card, CardFooter, CardHeader, CardPreview,
    Flex, FlexJustify, Icon, Select, Slider, Switch,
};
use crate::{
    animation::{AnimationControls, AnimationOptions, AnimationPlayer, FrameReel, PlayState},
    error::AdicShapeResult,
    leptos::{
        adic_component::basic::ComponentDisplay,
        util::mount_style,
        ShapeComponent,
    },
};



#[component]
/// Animated shape leptos card
///
/// ```no_run
/// # use adic::EAdic;
/// # use adic_shape::{
/// #     animation::FrameReel,
/// #     leptos::AnimatedShapeCard,
/// #     shape::{AdicCanvas, ClockCanvas, TreeCanvas},
/// # };
/// # use leptos::prelude::*;
/// let neg_1_4 = EAdic::new_repeating(5, vec![], vec![1]);
/// let neg_1_2 = neg_1_4.clone() + neg_1_4.clone();
/// let neg_3_4 = neg_1_2.clone() + neg_1_4.clone();
/// let neg_1 = neg_1_2.clone() + neg_1_2.clone();
/// let num_digits = 6;
///
/// let clock_canvas = ClockCanvas::builder().base(5).depth(num_digits).build();
/// let clock_frames = [
///     clock_canvas.draw_integer(&neg_1_4)?,
///     clock_canvas.draw_integer(&neg_1_2)?,
///     clock_canvas.draw_integer(&neg_3_4)?,
///     clock_canvas.draw_integer(&neg_1)?,
/// ];
/// let tree_canvas = TreeCanvas::builder().base(5).depth(num_digits).build();
/// let tree_frames = [
///     tree_canvas.draw_integer(&neg_1_4)?,
///     tree_canvas.draw_integer(&neg_1_2)?,
///     tree_canvas.draw_integer(&neg_3_4)?,
///     tree_canvas.draw_integer(&neg_1)?,
/// ];
/// let clocks = FrameReel::simple_linear(clock_frames);
/// let trees = FrameReel::simple_linear(tree_frames);
///
/// let animated_clock_view = view! {
///     <AnimatedShapeCard shape_reel=clocks/>
/// };
/// let animated_tree_view = view! {
///     <AnimatedShapeCard shape_reel=trees/>
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn AnimatedShapeCard<S>(
    #[prop(into)]
    /// Adic shape reel to display sequentially in SVG
    shape_reel: Signal<FrameReel<AdicShapeResult<S>>>,
    #[prop(into, optional)]
    /// Animation options
    options: RwSignal<AnimationOptions>,
    #[prop(into, optional)]
    /// Card HTML class
    class: Signal<Option<String>>,
    #[prop(into, optional)]
    /// Card title
    title: Signal<String>,
) -> impl IntoView
where S: Debug + Clone + Send + Sync + ComponentDisplay + 'static {

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    // Set up controls
    let controls = RwSignal::new(AnimationControls::default());

    // Animation player
    let player = RwSignal::new({
        let mut p = AnimationPlayer::new(shape_reel.get(), options.get_untracked().should_loop);
        p.start();
        p
    });


    // Trigger for forcing the frame of the animation
    let force_frame = Trigger::new();

    // Triggers for skipping forward and back in the animation
    let skip_back = Trigger::new();
    let skip_fwd = Trigger::new();

    // Animation triggers
    let triggers = AnimationTriggers {
        force_frame,
        skip_fwd,
        skip_back,
    };


    // Animation outer card
    let class = Signal::derive(move || [
        class.get().unwrap_or_default(),
        " shape-card".to_string()
    ].concat());
    view! {
        <Card class=class>

            <CardHeader class="shape-card-header">
                <h3 class="full-width center-text">{ move || title.get() }</h3>
            </CardHeader>

            <CardPreview class="shape-card-preview">
                <AnimatedShapeComponent player=player controls=controls triggers=triggers options=options/>
            </CardPreview>

            <CardFooter class="shape-card-footer">
                <Body1 class="full-width">
                    <Flex vertical=true>
                        <AnimatedShapeFrameLabel player=player options=options/>
                        <AnimatedShapeSlider player=player controls=controls triggers=triggers options=options/>
                        <AnimatedShapeControls player=player controls=controls triggers=triggers options=options/>
                    </Flex>
                </Body1>
            </CardFooter>

        </Card>
    }

}



#[component]
fn AnimatedShapeComponent<S>(
    #[prop(into)]
    /// Animation player, the state of the animation
    player: RwSignal<AnimationPlayer<AdicShapeResult<S>>>,
    #[prop(into)]
    /// Animation controls, e.g. play/pause and time between ticks
    controls: RwSignal<AnimationControls>,
    #[prop(into, optional)]
    /// Animation triggers, e.g. force animation to a frame or skip forward or back
    triggers: AnimationTriggers,
    #[prop(into, optional)]
    /// Animation options
    options: Signal<AnimationOptions>,
) -> impl IntoView
where S: Debug + Clone + Send + Sync + ComponentDisplay + 'static {

    let player_min_time_memo = Memo::new(move |_| player.get().min_time());

    // If should_auto_start is changed, reset the animation and switch the starting play state
    let auto_start_memo = Memo::new(move |_| options.get().should_auto_start);
    Effect::watch(
        move || (auto_start_memo.get(), player_min_time_memo.get()),
        move |(should_auto_start, min_time), _, _| {
            let mut c = controls.write();
            c.current_time = *min_time;
            if *should_auto_start {
                c.play_state = PlayState::Playing;
            } else {
                c.play_state = PlayState::Paused;
            }
            triggers.force_frame.notify();
        },
        true
    );

    // If should_loop is changed, reset the animation player and set the new looping behavior
    let should_loop_memo = Memo::new(move |_| options.get().should_loop);
    Effect::watch(
        move || (should_loop_memo.get(), player_min_time_memo.get()),
        move |(should_loop, min_time), _, _| {
            let (mut c, mut p) = (controls.write(), player.write());
            c.current_time = *min_time;
            c.play_state = PlayState::Paused;
            p.set_looping(*should_loop);
            p.start();
            triggers.force_frame.notify();
        },
        false
    );

    // Animation interval handle
    let interval_handle: RwSignal<Option<IntervalHandle>> = RwSignal::new(None);

    // Animation play effect
    Effect::new(move || {

        let tick_time = options.get().tick_time;
        match controls.get().play_state {
            // Don't track interval_handle; it is used as an effect only
            PlayState::Playing => untrack(move || {

                if let Some(handle) = interval_handle.get() {
                    handle.clear();
                }
                interval_handle.set(None);

                if player.get().is_completed() {
                    controls.write().play_state = PlayState::Paused;
                } else {

                    let handle = set_interval_with_handle(
                        move || {

                            // Tick
                            player.write().tick();

                            // Match controls time to player time
                            if let Some(current_time) = player.get().current_time() {
                                controls.write().current_time = current_time;
                            }

                        },
                        tick_time
                    );
                    match handle {
                        Ok(h) => { interval_handle.set(Some(h)) },
                        Err(err) => { panic!("Js error: {err:?}") },
                    }

                }

            }),
            // Don't track interval_handle; it is used as an effect only
            PlayState::Paused => untrack(move || {

                if let Some(handle) = interval_handle.get() {
                    handle.clear();
                }
                interval_handle.set(None);

            }),
        }

    });

    // If player completes, pause the controls and cancel and unset interval_handle
    let player_completion_memo = Memo::new(move |_| player.get().is_completed());
    Effect::watch(
        move || player_completion_memo.get(),
        move |player_completion, _, _| {
            if *player_completion {
                let (mut c, mut ih) = (controls.write(), interval_handle.write());
                c.play_state = PlayState::Paused;
                if let Some(h) = *ih {
                    h.clear();
                }
                *ih = None;
            }
        },
        false
    );

    // If this component is dropped, pause the controls and cancel and unset interval_handle
    Owner::on_cleanup(move || {
        let (mut c, mut ih) = (controls.write(), interval_handle.write());
        c.play_state = PlayState::Paused;
        if let Some(h) = *ih {
            h.clear();
        }
        *ih = None;
    });

    // Force switch to a frame in the player, at current controls time
    Effect::watch(
        move || triggers.force_frame.track(),
        move |(), _, _| {
            let (c, mut p) = (controls.write(), player.write());
            p.set_frame_from_time(c.current_time);
        },
        false
    );

    // Skip forward a frame
    Effect::watch(
        move || triggers.skip_fwd.track(),
        move |(), _, _| {
            let (mut c, mut p) = (controls.write(), player.write());
            p.tick();
            if let Some(t) = p.current_time() {
                c.current_time = t;
            }
        },
        false
    );

    // Skip backward a frame
    Effect::watch(
        move || triggers.skip_back.track(),
        move |(), _, _| {
            let (mut c, mut p) = (controls.write(), player.write());
            p.tick_back();
            if let Some(t) = p.current_time() {
                c.current_time = t;
            }
        },
        false
    );

    // Show animation or replacement text
    view! {
        <Show
            when=move || player.get().is_started()
            fallback=move || view! {
                <p class="anim-component-fallback">"Animation starting..."</p>
            }
        >
            {move || match player.get().current_data() {
                Some(s) => Either::Left(view! {
                    <ShapeComponent shape=s.clone() class="anim-component"/>
                }),
                None => Either::Right(view! {
                    <p class="anim-component-fallback">{"No shape to show".to_string()}</p>
                }),
            }}
        </Show>
    }

}


#[component]
fn AnimatedShapeFrameLabel<S>(
    #[prop(into)]
    /// Animation player, the state of the animation
    player: RwSignal<AnimationPlayer<AdicShapeResult<S>>>,
    #[prop(into, optional)]
    /// Animation options
    options: Signal<AnimationOptions>,
) -> impl IntoView
where S: Debug + Clone + Send + Sync + ComponentDisplay + 'static {

    // Show the frame label below the animation
    let show_label_memo = Memo::new(move |_| options.get().show_frame_label);
    let current_label = Memo::new(move |_| {
        let mut current_label = player.get().current_label().unwrap_or("").to_string();
        // If only whitespace, put in a zero-width unicode character to retain the label height
        if current_label.chars().all(char::is_whitespace) {
            current_label = '\u{200b}'.to_string();
        }
        current_label
    });
    view! {
        <Show
            when=move || show_label_memo.get()
            fallback=|| ()
        >
            <h3 class="full-width center-text">{move || current_label.get()}</h3>
        </Show>
    }

}


#[component]
fn AnimatedShapeSlider<S>(
    #[prop(into)]
    /// Animation player, the state of the animation
    player: RwSignal<AnimationPlayer<AdicShapeResult<S>>>,
    #[prop(into)]
    /// Animation controls, e.g. play/pause and time between ticks
    controls: RwSignal<AnimationControls>,
    #[prop(into, optional)]
    /// Animation triggers, e.g. force animation to a frame or skip forward or back
    triggers: AnimationTriggers,
    #[prop(into, optional)]
    /// Animation options
    options: Signal<AnimationOptions>,
) -> impl IntoView
where S: Debug + Clone + Send + Sync + ComponentDisplay + 'static {

    // Slider to display and control animation
    let slider_val = Signal::derive(move || f64::from(controls.get().current_time));
    let set_slider_val = SignalSetter::map(move |val: f64| {
        controls.write().current_time = u32::from_f64(val.round()).unwrap();
    });
    let slider_model = (slider_val, set_slider_val);
    let slider_steps = Signal::stored(1.0);
    let slider_min = Signal::derive(move || f64::from(player.get().min_time()));
    let slider_max = Signal::derive(move || f64::from(player.get().max_time()));

    let on_slider_input = move |_| {
        controls.write().play_state = PlayState::Paused;
        triggers.force_frame.notify();
    };

    // Show the slider below the animation
    let show_slider_memo = Memo::new(move |_| options.get().show_slider);
    view! {
        <Show
            when=move || show_slider_memo.get()
            fallback=|| ()
        >
            <Slider
                step=slider_steps min=slider_min max=slider_max value=slider_model
                on:input=on_slider_input
            />
        </Show>
    }

}


#[component]
fn AnimatedShapeControls<S>(
    #[prop(into)]
    /// Animation player, the state of the animation
    player: RwSignal<AnimationPlayer<AdicShapeResult<S>>>,
    #[prop(into)]
    /// Animation controls, e.g. play/pause and time between ticks
    controls: RwSignal<AnimationControls>,
    #[prop(into, optional)]
    /// Animation triggers, e.g. force animation to a frame or skip forward or back
    triggers: AnimationTriggers,
    #[prop(into, optional)]
    /// Animation options
    options: RwSignal<AnimationOptions>,
) -> impl IntoView
where S: Debug + Clone + Send + Sync + ComponentDisplay + 'static {

    // The icon for the play/pause button control
    let play_pause_icon = move || {
        match controls.get().play_state {
            PlayState::Playing => view!{ <Icon icon=icondata_fi::FiPause/> },
            PlayState::Paused => view!{ <Icon icon=icondata_fi::FiPlay/> },
        }
    };

    // Reset animation to beginning and pause
    let on_reset = move |_| {
        controls.write().current_time = player.get().min_time();
        controls.write().play_state = PlayState::Paused;
        triggers.force_frame.notify();
    };

    // Toggling play/pause
    let on_play_pause = move |_| {
        controls.write().play_state.toggle();
    };

    // Skip back and forth frames
    let on_skip_back = move |_| {
        triggers.skip_back.notify();
    };
    let on_skip_fwd = move |_| {
        triggers.skip_fwd.notify();
    };

    // Show the play/reset button control
    let show_play_reset_memo = Memo::new(move |_| options.get().show_play_reset);
    // Show the skip fwd/back button control
    let show_skip_memo = Memo::new(move |_| options.get().show_skip);

    // Controls below the animation
    view! {
        <Flex justify=FlexJustify::Center>
            <Show when=move || show_play_reset_memo.get() fallback=|| ()>
                <Button class="animation-control-button"
                    appearance=ButtonAppearance::Transparent
                    on_click=on_reset
                >
                    <Icon icon=icondata_fi::FiRotateCcw/>
                </Button>
                <Button class="animation-control-button"
                    appearance=ButtonAppearance::Transparent
                    on_click=on_play_pause
                >
                    {play_pause_icon}
                </Button>
            </Show>

            <Show when=move || show_skip_memo.get() fallback=|| ()>
                <Button class="animation-control-button"
                    appearance=ButtonAppearance::Transparent
                    on_click=on_skip_back
                >
                    <Icon icon=icondata_fi::FiSkipBack/>
                </Button>
                <Button class="animation-control-button"
                    appearance=ButtonAppearance::Transparent
                    on_click=on_skip_fwd
                >
                    <Icon icon=icondata_fi::FiSkipForward/>
                </Button>
            </Show>

            <Button class="animation-control-button"
                appearance=ButtonAppearance::Transparent
                attr:popovertarget="animated-shape-settings" attr:popovertargetaction="toggle"
            >
                <Icon icon=icondata_fi::FiSettings/>
            </Button>

        </Flex>

        <div id="animated-shape-settings" popover="auto">
            <AnimatedShapeSettings options=options/>
        </div>

    }
}


#[component]
fn AnimatedShapeSettings(
    #[prop(into)]
    /// Options modified by this settings menu
    options: RwSignal<AnimationOptions>,
) -> impl IntoView {

    macro_rules! map_signal {
        ( $option_attr:ident ) => {
            (
                Signal::derive(move || options.get().$option_attr),
                SignalSetter::map(move |new_val| options.write().$option_attr = new_val)
            )
        }
    }

    let should_auto_start = map_signal!(should_auto_start);
    let should_loop = map_signal!(should_loop);
    let show_slider = map_signal!(show_slider);
    let show_play_reset = map_signal!(show_play_reset);
    let show_skip = map_signal!(show_skip);
    let show_frame_label = map_signal!(show_frame_label);

    let tick_time_select = RwSignal::new("2 fps".to_string());
    let tick_time = (
        Signal::derive(move || options.get().tick_time),
        SignalSetter::<Duration>::map(move |new_val| options.write().tick_time = new_val)
    );
    Effect::watch(
        move || tick_time_select.get(),
        move |select, _, _| {
            tick_time.1.set(match select.as_ref() {
                "20 fps" => Duration::from_millis(50),
                "10 fps" => Duration::from_millis(100),
                "5 fps" => Duration::from_millis(200),
                "2 fps" => Duration::from_millis(500),
                "1 fps" => Duration::from_secs(1),
                _ => Duration::from_secs(1),
            });
        },
        true
    );

    view! {
        <Flex vertical=true class="shape-card-settings-menu">
            <Select value=tick_time_select default_value="2 fps">
                <option>"20 fps"</option>
                <option>"10 fps"</option>
                <option>"5 fps"</option>
                <option>"2 fps"</option>
                <option>"1 fps"</option>
            </Select>
            <Switch checked=should_auto_start label="Auto-start"/>
            <Switch checked=should_loop label="Looping"/>
            <Switch checked=show_frame_label label="Show frame label"/>
            <Switch checked=show_slider label="Show slider"/>
            <Switch checked=show_play_reset label="Show play/reset"/>
            <Switch checked=show_skip label="Show skip"/>
        </Flex>
    }

}


#[derive(Debug, Clone, Copy, Default)]
// Triggers for animation communication
struct AnimationTriggers {
    pub force_frame: Trigger,
    pub skip_fwd: Trigger,
    pub skip_back: Trigger,
}
