use num::Rational32;
use leptos::prelude::*;
use thaw::Flex;

use adic::{
    traits::{AdicPrimitive, HasApproximateDigits, PrimedFrom, TryPrimedFrom},
    EAdic, Polynomial, QAdic, ZAdic,
};
use adic_shape::{
    error::AdicShapeError,
    leptos::{Collapse, ShapeCard, ShapeComponent},
    shape::{AdicCanvas, Direction, TreeCanvas, TreeShape},
};


#[component]
pub fn TreeSuite() -> impl IntoView {

    let basic_canvas = Signal::derive(move || TreeCanvas::builder().base(5).depth(6).build());
    let basic = view! {
        <Collapse title="Basic">

            <Flex>
                <ShapeCard class="tree"
                    title="1 = ...000001"
                    shape=basic_canvas.get().draw_integer(&EAdic::primed_from(5, 1))
                />
                <ShapeCard class="tree"
                    title="-1 = ...444444"
                    shape=basic_canvas.get().draw_integer(&EAdic::primed_from(5, -1))
                />
                <ShapeCard class="tree"
                    title="-1/4 = ...111111"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-1, 4)).unwrap())
                />
                <ShapeCard class="tree"
                    title="-1/24 = ...010101"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-1, 24)).unwrap())
                />
                <ShapeCard class="tree"
                    title="-5/24 = ...101010"
                    shape=basic_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
                />
            </Flex>

        </Collapse>
    };

    let fractional_canvas = Signal::derive(move || TreeCanvas::builder().base(5).depth(6).build());
    let fractional = view! {
        <Collapse title="Fractional">

            <Flex>
                <ShapeCard class="tree"
                    title="1/5 = ...000000.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, 1), -1))
                />
                <ShapeCard class="tree"
                    title="-1/5 = ...444444.4"
                    shape=fractional_canvas.get().draw_number(&QAdic::new(EAdic::primed_from(5, -1), -1))
                />
                <ShapeCard class="tree"
                    title="-1/20 = ...111111.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 20)))
                />
                <ShapeCard class="tree"
                    title="-1/120 = ...101010.1"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-1, 120)))
                />
                <ShapeCard class="tree"
                    title="-5/120 = ...010101.0"
                    shape=fractional_canvas.get().draw_number(&QAdic::<EAdic>::primed_from(5, Rational32::new(-5, 120)))
                />
            </Flex>

        </Collapse>
    };

    let comp_vs_card_canvas = Signal::derive(move || TreeCanvas::builder().base(5).depth(6).build());
    let component_vs_card = view! {
        <Collapse title="Component vs Card">
            <h2>"Component"</h2>
            <ShapeComponent class="tree"
                shape=comp_vs_card_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
            />
            <h2>"Card"</h2>
            <ShapeCard class="tree"
                title="Tree title message"
                shape=comp_vs_card_canvas.get().draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
            />
        </Collapse>
    };

    let sqrt2_7_canvas = Signal::derive(move || TreeCanvas::builder().base(7).depth(20).build());
    let sqrt2_7adic = view! {
        <Collapse title="\\sqrt{2} in the 7-adics">

            <Flex>
                <ShapeCard class="tree"
                    title="2"
                    shape=sqrt2_7_canvas.get().draw_integer(&ZAdic::new_approx(7, 20, vec![2]))
                />
                <ShapeCard class="tree"
                    title="1st \\sqrt{2}"
                    shape=sqrt2_7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![3, 1, 2, 6, 1, 2, 1, 2, 4, 6, 6, 2, 1, 1, 0, 2, 1, 1, 4])
                    )
                />
                <ShapeCard class="tree"
                    title="2nd \\sqrt{2}"
                    shape=sqrt2_7_canvas.get().draw_integer(
                        &ZAdic::new_approx(7, 20, vec![4, 5, 4, 0, 5, 4, 5, 4, 2, 0, 0, 4, 5, 5, 6, 4, 5, 5, 2])
                    )
                />
            </Flex>

        </Collapse>
    };

    let sqrt7_3_canvas = Signal::derive(move || TreeCanvas::builder().base(3).depth(20).build());
    let sqrt7_3adic = view! {
        <Collapse title="\\sqrt{7} in the 3-adics">

            <Flex>
                <ShapeCard class="tree"
                    title="Root 1"
                    shape=sqrt7_3_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![1, 1, 1, 0, 2, 0, 0, 2, 1, 1, 2, 0, 2, 2, 2, 1, 0, 2, 1, 2])
                    )
                />
                <ShapeCard class="tree"
                    title="Root 2"
                    shape=sqrt7_3_canvas.get().draw_integer(
                        &ZAdic::new_approx(3, 20, vec![2, 1, 1, 2, 0, 2, 2, 0, 1, 1, 0, 2, 0, 0, 0, 1, 2, 0, 1, 0])
                    )
                />
            </Flex>

        </Collapse>
    };

    let split_tree = view! {
        <Collapse title="Split tree">
            <Flex>
                <ShapeCard
                    class="tree" title="5-adic 0 and -1"
                    shape=make_split_tree(&[EAdic::primed_from(5, 0), EAdic::primed_from(5, -1)], 5, 4)
                />
                <ShapeCard
                    class="tree" title="11-adic 0 and -1"
                    shape=make_split_tree(&[EAdic::primed_from(11, 0), EAdic::primed_from(11, -1)], 11, 4)
                />
            </Flex>
        </Collapse>
    };

    let one_roots = view! {
        <Collapse title="Roots of unity">

            <ShapeCard class="tree" title="2-adic roots of unity"
                shape=make_split_tree(ZAdic::roots_of_unity(2, 10).unwrap().roots(), 2, 10)
            />
            <ShapeCard class="tree" title="3-adic roots of unity"
                shape=make_split_tree(ZAdic::roots_of_unity(3, 10).unwrap().roots(), 3, 10)
            />
            <ShapeCard class="tree" title="5-adic roots of unity"
                shape=make_split_tree(ZAdic::roots_of_unity(5, 10).unwrap().roots(), 5, 10)
            />
            <ShapeCard class="tree" title="7-adic roots of unity"
                shape=make_split_tree(ZAdic::roots_of_unity(7, 10).unwrap().roots(), 7, 10)
            />
            <ShapeCard class="tree" title="11-adic roots of unity"
                shape=make_split_tree(ZAdic::roots_of_unity(11, 10).unwrap().roots(), 11, 10)
            />

        </Collapse>
    };

    let polynomial = Polynomial::<EAdic>::new_with_prime(5, vec![138, -109, 18, 1]);
    let variety = polynomial.variety(10).unwrap().try_into_integer().unwrap();
    let shape_triple_root = make_split_tree(variety.roots(), 5, 6).unwrap();

    let polynomial = Polynomial::<EAdic>::new_with_prime(5, vec![0, 24, -50, 35, -10, 1]);
    let variety = polynomial.variety(10).unwrap().try_into_integer().unwrap();
    let shape_quint_root = make_split_tree(variety.roots(), 5, 6).unwrap();

    let hensel_examples = view! {
        <Collapse title="Hensel examples">
            <Flex>
                <ShapeCard class="tree" title="Hensel (x+23)(x-3)(x-2) = x^3 + 18 x^2 - 109 x + 138"
                    shape=shape_triple_root
                />
                <ShapeCard class="tree" title="Hensel x(x-1)(x-2)(x-3)(x-4) = x^5 - 10 x^4 + 35 x^3 - 50 x^2 + 24 x"
                    shape=shape_quint_root
                />
            </Flex>
        </Collapse>
    };

    let orientation = view! {
        <Collapse title="Orientation">

            <Flex>
                <ShapeCard class="tree" title="Up" shape=make_split_roots_dir(Direction::Up)/>
                <ShapeCard class="tree" title="Down" shape=make_split_roots_dir(Direction::Down)/>
                <ShapeCard class="tree" title="Left" shape=make_split_roots_dir(Direction::Left)/>
                <ShapeCard class="tree" title="Right" shape=make_split_roots_dir(Direction::Right)/>
            </Flex>

            <Flex>
                <ShapeCard class="tree" title="Up" shape=make_split_01_dir(Direction::Up)/>
                <ShapeCard class="tree" title="Down" shape=make_split_01_dir(Direction::Down)/>
                <ShapeCard class="tree" title="Left" shape=make_split_01_dir(Direction::Left)/>
                <ShapeCard class="tree" title="Right" shape=make_split_01_dir(Direction::Right)/>
            </Flex>

        </Collapse>
    };

    let full_trees = view! {
        <Collapse title="Full trees">

            <Flex>
                <ShapeCard class="tree"
                    title="Full 5-adic tree"
                    shape=TreeCanvas::builder().base(5).depth(4).solid_full_tree().build().draw_full()
                />
                <ShapeCard class="tree"
                    title="Dashed 5-adic tree"
                    shape=TreeCanvas::builder().base(5).depth(4).dashed_full_tree().build().draw_full()
                />
            </Flex>

            <Flex>
                <ShapeCard class="tree"
                    title="Full 5-adic tree"
                    shape=TreeCanvas::builder().base(5).depth(4).solid_full_tree().build().draw_integer(&EAdic::zero(5))
                />
                <ShapeCard class="tree"
                    title="Dashed 5-adic tree"
                    shape=TreeCanvas::builder().base(5).depth(4).dashed_full_tree().build().draw_integer(&EAdic::zero(5))
                />
            </Flex>

        </Collapse>
    };

    let adic_trees = view! {
        <Collapse title="Full colored trees">

            <Flex>
                <ShapeCard class="tree"
                    title="1 = ...0001"
                    shape=make_adic_tree(&EAdic::primed_from(5, 1), 3)
                />

                <ShapeCard class="tree"
                    title="-1 = ...444444"
                    shape=make_adic_tree(&EAdic::primed_from(5, -1), 3)
                />
                <ShapeCard class="tree"
                    title="-1/4 = ...111111"
                    shape=make_adic_tree(&EAdic::try_primed_from(5, Rational32::new(-1, 4)).unwrap(), 3)
                />
            </Flex>

            <h2>"\\sqrt{7} in the 3-adics"</h2>

            <Flex>
                <ShapeCard class="tree"
                    title="Root 1"
                    shape=make_adic_tree(&ZAdic::new_approx(3, 5, vec![1, 1, 1, 0, 2]), 5)
                />
                <ShapeCard class="tree"
                    title="Root 2"
                    shape=make_adic_tree(&ZAdic::new_approx(3, 5, vec![2, 1, 1, 2, 0]), 5)
                />
            </Flex>

            <h4>"Roots of unity"</h4>

            <Flex>
                <ShapeCard class="tree"
                    shape=make_adic_tree(&ZAdic::new_approx(5, 3, vec![1]), 3)
                />
                <ShapeCard class="tree"
                    shape=make_adic_tree(&ZAdic::new_approx(5, 3, vec![2, 1, 2]), 3)
                />
                <ShapeCard class="tree"
                    shape=make_adic_tree(&ZAdic::new_approx(5, 3, vec![3, 3, 2]), 3)
                />
                <ShapeCard class="tree"
                    shape=make_adic_tree(&ZAdic::new_approx(5, 3, vec![4, 4, 4]), 3)
                />
            </Flex>

        </Collapse>
    };

    let tree_result = view! {
        <Collapse title="Tree result component">

            <Flex>
                <ShapeCard class="tree debug-outline"
                    title="Ok tree"
                    shape=TreeCanvas::builder()
                        .base(5).depth(3)
                        .solid_full_tree()
                        .build()
                        .draw_integer(&EAdic::try_primed_from(5, Rational32::new(-5, 24)).unwrap())
                />
                <ShapeCard<TreeShape> class="tree debug-outline"
                    title="Err tree"
                    shape=Err(AdicShapeError::ImproperConfig("Something didn't work".to_string()))
                />
            </Flex>

        </Collapse>
    };

    view! {
        <section class="boxed-section">
            {basic}
            {fractional}
            {component_vs_card}
            {sqrt2_7adic}
            {sqrt7_3adic}
            {split_tree}
            {one_roots}
            {hensel_examples}
            {orientation}
            {full_trees}
            {adic_trees}
            {tree_result}
        </section>
    }

}


fn make_split_tree<'a, A, IA>(ia: IA, base: u32, depth: isize) -> Result<TreeShape, AdicShapeError>
where IA: IntoIterator<Item = &'a A>, A: Clone + HasApproximateDigits<DigitIndex = usize> + 'a {
    TreeCanvas::builder().base(base).depth(depth).build().draw_integers(ia)
}

fn make_split_roots_dir(direction: Direction) -> Result<TreeShape, AdicShapeError> {
    let dangling_direction = Some(direction.cwise());
    let variety = ZAdic::roots_of_unity(5, 10).unwrap();
    let ia = variety.roots();
    TreeCanvas::builder()
        .base(5).depth(10)
        .direction(direction).dangling_direction(dangling_direction)
        .build()
        .draw_integers(ia)
}
fn make_split_01_dir(direction: Direction) -> Result<TreeShape, AdicShapeError> {
    let dangling_direction = Some(direction.cwise());
    let ia = [EAdic::primed_from(5, 0), EAdic::primed_from(5, -1)];
    TreeCanvas::builder()
        .base(5).depth(4)
        .direction(direction).dangling_direction(dangling_direction)
        .build()
        .draw_integers(&ia)
}

fn make_adic_tree<A>(a: &A, depth: isize) -> Result<TreeShape, AdicShapeError>
where A: Clone + HasApproximateDigits<DigitIndex = usize> {
    TreeCanvas::builder().base(a.base().into()).depth(depth).solid_full_tree().build().draw_integer(a)
}
