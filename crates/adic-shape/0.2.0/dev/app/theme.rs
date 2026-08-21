use std::{collections::HashMap, str::FromStr};
use leptos::prelude::*;
use num::FromPrimitive;
use palette::{Darken, IntoColor, Lighten, Mix, Srgb};
use thaw::{
    Button, ButtonAppearance, Card, CardHeader,
    Flex, InfoLabel, InfoLabelInfo, Theme,
};

use adic::{
    traits::AdicInteger,
    EAdic, ZAdic,
};
use adic_shape::{
    leptos::{Collapse, ShapeCard},
    shape::{AdicCanvas, ClockCanvas, EuclideanCanvas, TreeCanvas},
};


pub use thaw::ConfigProvider;


pub fn default_theme() -> Theme {
    default_dark_theme()
}

pub fn default_light_theme() -> Theme {
    let color = thaw::Color::RGB(Srgb::from_str("#916949").unwrap().into_format());
    light_theme(&color)
}

pub fn default_dark_theme() -> Theme {
    let color = thaw::Color::RGB(Srgb::from_str("#916949").unwrap().into_format());
    dark_theme(&color)
}

pub fn light_theme(color: &thaw::Color) -> Theme {
    let hash_map = create_brand_colors(color);
    let mapped = hash_map.iter().map(|(k, v)| (*k, v.as_str())).collect();
    Theme::custom_light(&mapped)
}

pub fn dark_theme(color: &thaw::Color) -> Theme {
    let hash_map = create_brand_colors(color);
    let mapped = hash_map.iter().map(|(k, v)| (*k, v.as_str())).collect();
    Theme::custom_dark(&mapped)
}


#[component]
pub fn ThemeControl(
    global_theme: RwSignal<Theme>,
) -> impl IntoView {

    let local_theme = RwSignal::new(default_theme());

    let on_default_light_theme = move |_| {
        local_theme.set(default_light_theme());
    };
    let on_default_dark_theme = move |_| {
        local_theme.set(default_dark_theme());
    };

    let color_pick = RwSignal::new(thaw::Color::from(palette::Srgb::new(0.0, 0.0, 0.0)));
    let on_custom_light_theme = move |_| {
        let c = color_pick.get();
        local_theme.set(light_theme(&c));
    };
    let on_custom_dark_theme = move |_| {
        let c = color_pick.get();
        local_theme.set(dark_theme(&c));
    };

    let on_apply_globally = move |_| {
        global_theme.set(local_theme.get());
    };

    let theme_control = view!{
        <Card>

            <Card>
                <CardHeader>"Default theme"</CardHeader>
                <Flex style:align-self="center">
                    <Button appearance=ButtonAppearance::Primary on_click=on_default_light_theme>"Default Light Theme"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_default_dark_theme>"Default Dark Theme"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_apply_globally>"Apply globally"</Button>
                </Flex>
            </Card>

            <Card>
                <CardHeader>"Custom color theme"</CardHeader>
                <Flex style:align-self="center">
                    <thaw::ColorPicker size=thaw::ColorPickerSize::Small style:flex="1" value=color_pick/>
                    <Button appearance=ButtonAppearance::Primary on_click=on_custom_light_theme>"Custom Light Theme"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_custom_dark_theme>"Custom Dark Theme"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_apply_globally>"Apply globally"</Button>
                </Flex>
            </Card>

        </Card>
    };

    view! {
        <ConfigProvider theme=local_theme>
            {theme_control}
            <AdicShapeShowcase/>
            <ThawElementShowcase/>
        </ConfigProvider>
    }

}


#[component]
fn AdicShapeShowcase() -> impl IntoView {

    let neg_1_4 = EAdic::new_repeating(5, vec![], vec![1]);
    let sqrt_2 = EAdic::new(7, vec![2]).nth_root(2, 6).unwrap().into_roots().next().unwrap();
    let clock_canvas = ClockCanvas::builder().base(5).depth(6).build();
    let clock0 = clock_canvas.draw_integer(&neg_1_4);
    let clock_canvas = ClockCanvas::builder().base(7).depth(6).build();
    let clock1 = clock_canvas.draw_integer(&sqrt_2);
    let tree_canvas = TreeCanvas::builder().base(5).depth(6).build();
    let tree0 = tree_canvas.draw_integer(&neg_1_4);
    let tree_canvas = TreeCanvas::builder().base(7).depth(6).build();
    let tree1 = tree_canvas.draw_integer(&sqrt_2);
    let tree_canvas = TreeCanvas::builder().base(5).depth(3).solid_full_tree().build();
    let full_tree0 = tree_canvas.draw_integer(&neg_1_4);
    let tree_canvas = TreeCanvas::builder().base(3).depth(5).solid_full_tree().build();
    let full_tree1 = tree_canvas.draw_full();
    let euclidean_scaled_hulls_canvas = EuclideanCanvas::builder()
        .fixed_hulls(vec![(0.2, 0.0), (0.8, 0.0), (1.0, 0.6), (0.5, 1.0), (0.0, 0.6)])
        .scaling(3.0).depth(3)
        .solid_full_tree()
        .draw_scaled_hulls()
        .build();
    let euclidean0 = euclidean_scaled_hulls_canvas.draw_full().unwrap();
    let teich = ZAdic::teichmuller(5, 3).unwrap();
    let euclidean_characteristic_canvas = EuclideanCanvas::builder()
        .characteristic_p_adic(5)
        .scaling(2.8).depth(3)
        .draw_scaled_dots()
        .draw_enclosing_disks([0, 1, 2])
        .build();
    let euclidean1 = euclidean_characteristic_canvas.draw_integers(teich.roots()).unwrap();

    view!{
        <Collapse title="Adic shape showcase">

            <Flex>
                <ShapeCard class="clock" title="5-adic -1/4 Clock" shape=clock0/>
                <ShapeCard class="clock" title="7-adic sqrt(2) Clock" shape=clock1/>
            </Flex>

            <Flex>
                <ShapeCard class="tree" title="5-adic -1/4 Tree" shape=tree0/>
                <ShapeCard class="tree" title="7-adic sqrt(2) Tree" shape=tree1/>
            </Flex>

            <Flex>
                <ShapeCard class="tree" title="5-adic -1/4 Tree" shape=full_tree0/>
                <ShapeCard class="tree" title="Full 3-adic Tree" shape=full_tree1/>
            </Flex>

            <Flex>
                <ShapeCard class="euclidean" title="Off-center pentagon Euclidean" shape=euclidean0/>
                <ShapeCard class="euclidean" title="5-adic Teichmuller Euclidean" shape=euclidean1/>
            </Flex>

        </Collapse>
    }

}

#[component]
fn ThawElementShowcase() -> impl IntoView {

    let value = RwSignal::new(0.0);
    let page = RwSignal::new(1);
    let selected_value = RwSignal::new(String::from("apple"));

    view!{
        <Collapse title="Thaw element showcase">
            <Card>

                <thaw::Link href="https://react.fluentui.dev/?path=/docs/theme-theme-designer--docs">
                    "You can use this tool to generate brand color palette"
                </thaw::Link>

                <thaw::TabList selected_value>
                    <thaw::Tab value="apple">
                        "Apple"
                    </thaw::Tab>
                    <thaw::Tab value="pear">
                        "Pear"
                    </thaw::Tab>
                </thaw::TabList>

                <thaw::RadioGroup value=selected_value>
                    <thaw::Radio value="apple" label="Apple"/>
                    <thaw::Radio value="pear" label="Pear"/>
                </thaw::RadioGroup>

                <InfoLabel>
                    <InfoLabelInfo slot>
                        "This is example information for an InfoLabel. "
                    </InfoLabelInfo>
                    "Example label"
                </InfoLabel>

                <thaw::Badge appearance=thaw::BadgeAppearance::Filled>"10+"</thaw::Badge>
                <thaw::Checkbox />
                <thaw::Switch />
                <thaw::Tag dismissible=true>"Tag"</thaw::Tag> //TODO make dismissable
                <thaw::Input/>

                <thaw::Select>
                    <option>"Red"</option>
                    <option>"Green"</option>
                    <option>"Blue"</option>
                </thaw::Select>

                <thaw::TimePicker />
                <thaw::DatePicker/>

                <thaw::Slider step=25.0 value/>
                <thaw::ProgressBar value/>
                <thaw::Pagination page page_count=10 />

            </Card>
        </Collapse>
    }

}

fn create_brand_colors(init_color: &thaw::Color) -> HashMap<i32, String> {

    let color = match init_color.clone() {
        thaw::Color::RGB(rgb) => rgb.into_format().into_color(),
        thaw::Color::HSV(hsv) => hsv.into_format().into_color(),
        thaw::Color::HSL(hsl) => hsl,
    };

    let dark = color.darken(0.9);
    let bright = color.lighten(0.9);

    let colors = (1..9).map(|idx| {
        // Interpolate between color and dark
        let amount = f32::from_i32(9-idx).unwrap() / 8.0;
        color.mix(dark, amount)
    }).chain((0..8).map(|idx| {
        // Interpolate between color and dark
        let amount = f32::from_i32(idx).unwrap() / 7.0;
        color.mix(bright, amount)
    })).map(|c| {
        let c: Srgb = c.into_color();
        let c: Srgb<u8> = c.into_format();
        format!("#{c:X}")
    }).enumerate().map(|(idx, color_str)| (
        i32::try_from((idx+1) * 10).expect("usize -> i32 conversion"),
        color_str
    )).collect::<HashMap<_, _>>();

    colors

}
