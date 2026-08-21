
///
/// Calls "enter()" on the provided "$to_enter" parameter in a block, storing the result in an invariant. Then the provided "$expr" expression parameter is then run in this block.
/// 
#[macro_export]
macro_rules! enter
{

    ($to_enter:ident, $expr:expr) =>
    {
        
        {

            let _entered = $to_enter.enter();

            $expr

        }

    }

}

