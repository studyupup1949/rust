use adic::{
    divisible::Prime,
    normed::{UltraNormed, Valuation},
    traits::{AdicPrimitive, HasDigits},
    ZAdic,
};
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart, Interpolation, IntoEdge, IntoInner, Legend, Line, RotatedLabel,
    Series, Step, TickLabels, Tooltip, TooltipPlacement, XGuideLine,
};
use leptos::{prelude::*, logging::log};
use num::ToPrimitive;
use crate::{
    error::{AdicShapeError, AdicShapeResult},
    leptos::util::mount_style,
};


#[derive(Clone, Debug)]
struct Data {
    x: f64,
    y: f64,
    _adic_num: ZAdic,
}

#[component]
/// Project adic functions to the real line, usually creating nowhere-differentiable functions
///
/// ```no_run
/// # use adic::{traits::PrimedFrom, ZAdic};
/// # use adic_shape::leptos::RealProjectionChart;
/// # use leptos::prelude::*;
/// let range_xy = (1..15625).map(|integer| ZAdic::primed_from(5, integer)).collect::<Vec<_>>();
/// let adic_fn = |x: ZAdic| Ok(x.clone() * x.clone() * x);
/// let chart = view! {
///     <RealProjectionChart range=range_xy.clone() function=adic_fn title="x^3"/>
/// };
/// ```
pub fn RealProjectionChart(
    #[prop()]
    range: impl IntoIterator<Item = ZAdic>,
    #[prop()]
    function: impl Fn(ZAdic) -> AdicShapeResult<ZAdic>,
    #[prop(into, default = "".into())]
    title: Signal<String>,
    #[prop(into, default = "".into())]
    data_label: Signal<String>,
) -> impl IntoView {

    if let Err(err) = mount_style(
        "shape-component",
        include_str!("../shape-component.css")
    ) {
        log!("Error mounting css: {err:?}");
    }

    let (data, tooltip, bottom) = match get_data(range, function) {
        Ok((data, p)) => {

            let p64 = f64::from(u32::from(p));
            let num_digits = 6;
            let adic_labeller = TickLabels::aligned_floats().with_format(move |&tick, fmt| {
                let mut tick = tick;
                let mut adic_num = 0.0;
                for digit_place in 0..num_digits {
                    let itick = tick.trunc();
                    let mtick = tick.fract();
                    adic_num += 10f64.powf(f64::from(digit_place)) * itick;
                    tick = mtick * p64;
                }
                adic_num = adic_num.round();
                fmt.format(&adic_num)
            });

            let tooltip = Tooltip::new(
                TooltipPlacement::LeftCursor,
                adic_labeller.clone(),
                adic_labeller.clone()
            );

            (data, tooltip, adic_labeller.into_edge())

        },
        Err(err) => {

            let tooltip = Tooltip::left_cursor();
            let bottom = RotatedLabel::middle(err.to_string());

            (vec![], tooltip, bottom.into_edge())

        }
    };

    let component = move || {

        // Lines are added to the series
        let series = Series::<Data, f64, f64>::new(|data: &Data| data.x)
            .with_min_y(0.)
            .line(Line::new(move |data: &Data| data.y)
                .with_name(data_label.get())
                .with_interpolation(Interpolation::Step(Step::HorizontalMiddle)));

        view! {
            <Chart
                // Sets the width and height
                aspect_ratio=AspectRatio::from_outer_ratio(1200.0, 600.0)

                // Decorate our chart
                top=RotatedLabel::middle(title.get())
                right=Legend::end()
                bottom=vec![bottom.clone()]

                inner=[
                    AxisMarker::bottom_edge().into_inner(),
                    XGuideLine::over_data().into_inner(),
                ]
                tooltip=tooltip.clone()

                // Describe the data
                series=series
                data=data.clone()
            />
        }

    };

    view! {{ component }}
}


fn get_data(
    range: impl IntoIterator<Item = ZAdic>,
    function: impl Fn(ZAdic) -> AdicShapeResult<ZAdic>,
) -> AdicShapeResult<(Vec<Data>, Prime)> {

    let mut i = range.into_iter().peekable();
    let p = i.peek().map(AdicPrimitive::p).ok_or(AdicShapeError::Math("Data empty".to_string()))?;
    let mut data = i.map(|adic_num| {
        if adic_num.p() != p {
            Err(AdicShapeError::Math("Mixed characteristic; some adic numbers have different primes".to_string()))?;
        }

        let adic_output = function(adic_num.clone())?;

        let input_offset = match adic_num.valuation() {
            //If we want to support QAdics this is necessary
            //Valuation::Finite(v) if v < 0 => v,
            Valuation::Finite(_) => 0,
            Valuation::PosInf => Err(AdicShapeError::TooLarge("Valuation cannot be infinite for the input".to_string()))?,
        };
        let output_offset = match adic_output.valuation() {
            //If we want to support QAdics this is necessary
            //Valuation::Finite(v) if v < 0 => v,
            Valuation::Finite(_) => 0,
            Valuation::PosInf => Err(AdicShapeError::TooLarge("Valuation cannot be infinite for the output".to_string()))?,
        };

        let inverse_p = 1.0 / f64::from(u32::from(p));
        let x = adic_num.digits().enumerate().map(|(i, d)| {
            let power = (i + input_offset).to_f64().expect("usize -> f64 conversion");
            f64::from(d)*inverse_p.powf(power)
        }).sum();
        let y = adic_output.digits().enumerate().map(|(i, d)| {
            let power = (i + output_offset).to_f64().expect("usize -> f64 conversion");
            f64::from(d)*inverse_p.powf(power)
        }).sum();
        Ok(Data {
            _adic_num: adic_num,
            x,
            y,
        })
    }).collect::<AdicShapeResult<Vec<_>>>()?;
    data.sort_by(|a, b| a.x.total_cmp(&b.x));

    Ok((data, p))

}
