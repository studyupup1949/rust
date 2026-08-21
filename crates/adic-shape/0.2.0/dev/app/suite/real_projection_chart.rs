use num::traits::Pow;
use leptos::prelude::*;
use adic_shape::adic::{
    /*series::adic_digamma, series::adic_log, AdicSized,*/
    error::AdicError,
    traits::PrimedFrom,
    ZAdic,
};
use adic_shape::leptos::RealProjectionChart;


/// App entry point, hosts meta, router, fallback, and routes
#[component]
pub fn RealProjectionChartSuite() -> impl IntoView {

    //let range_digamma = (1..49).map(|integer| ZAdic::from_u32(7, integer)).collect::<Vec<_>>();
    //let adic_output_digamma = |adic_num| adic_digamma(adic_num, 4).unwrap();
    // let range_log = (1..2048).map(|integer| ZAdic::from_u32(2, integer)).collect::<Vec<_>>();
    // let range_log3 = (1..729).map(|integer| ZAdic::from_u32(3, integer)).collect::<Vec<_>>();
    // let range_log5 = (1..625).map(|integer| ZAdic::from_u32(5, integer)).collect::<Vec<_>>();
    // let range_log7 = (1..343).map(|integer| ZAdic::from_u32(7, integer)).collect::<Vec<_>>();
    let range_xy = (1..15625).map(|integer| ZAdic::primed_from(5, integer)).collect::<Vec<_>>();
    //let adic_output_log = |adic_num: ZAdic| Ok(adic_log(adic_num.clone(), 15)?.into_unit().unwrap_or(ZAdic::zero(adic_num.p())));


    //let spam_me = RwSignal::new(true);
    //let title = Signal::derive(move || if spam_me.get() {"Success"} else {"Failure"});

    view! {
        <main>
            //<input type="checkbox" bind:checked=spam_me/>
            //<RealProjectionChart range=range_digamma function=adic_output_digamma/>
            //<RealProjectionChart range=range_log function=adic_output_log title=title/>
            //<RealProjectionChart range=range_log3 function=adic_output_log title="Log(x)"/>
            //<RealProjectionChart range=range_log5 function=adic_output_log title="Log(x)"/>
            //<RealProjectionChart range=range_log7 function=adic_output_log title="Log(x)"/>
            <RealProjectionChart range=range_xy.clone() function={|x| Ok(x)} title="x"/>
            <RealProjectionChart range=range_xy.clone() function={|x| Ok(x.clone()*x)} title="x^2"/>
            <RealProjectionChart range=range_xy.clone() function={|x| Ok(x.pow(3))} title="x^3"/>
            <RealProjectionChart range=range_xy.clone() function={|x| Ok(x.pow(4))} title="x^4"/>
            <RealProjectionChart range=range_xy.clone() function={|x| Ok(x.pow(5))} title="x^5"/>
            <RealProjectionChart range=range_xy.clone() function={|_x| Err(AdicError::Severe("Severe".to_string()).into())} title="Error"/>
        </main>
    }


}
