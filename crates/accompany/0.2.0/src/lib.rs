#[macro_export] macro_rules! bound {
    (with $($patterned_identifier:pat = $expression:expr) , + =>$bound_code:block) => {
       {
           // patterned_identifier includes examples like tuple destruction, struct destruction and conventional identifier
           $(let $patterned_identifier = $expression;)+
           {$bound_code}
       }
    };
    (with $($patterned_identifier:pat = $expression:expr) , + =>$bound_code:expr) => {
       {
           // patterned_identifier includes examples like tuple destruction, struct destruction and conventional identifier
           $(let $patterned_identifier = $expression;)+
           {$bound_code}
       }
    }
}


#[cfg(test)]
mod macro_tests {
    #[derive(Copy, Clone)]
    struct A {
        pub field: u8,
        pub field_u16: u16,
    }

    struct B {
        pub field: u8,
    }

    #[test]
    fn test_with_bound() {
        let simple_result = bound! {
            with i = 3 => {
                let m = i+1;
                m
            }
        };
        assert_eq!(4, simple_result);

        let simple_result = bound! {
            with mut i = 3 => {
                i += 1;
                let m = i+1;
                m
            }
        };
        assert_eq!(5, simple_result);

        let a = A {
            field: 1,
            field_u16: 2,
        };
        let destruct_result = bound! {
            with A { field: i, field_u16: _j } = a.clone() =>{
                let m = i+1;
                m
            }
        };
        assert_eq!(2, destruct_result);

        let destruct_result = bound! {
            with A { field: mut i, field_u16: _j } = a =>{
                i += 1;
                let m = i+1;
                m
            }
        };
        assert_eq!(3, destruct_result);

        let detuple_result = bound! {
            with (mut i,_j)=(4,3) =>{
                i += 1;
                let m = i+1;
                m
            }
        };
        assert_eq!(6, detuple_result);
    }

    #[test]
    fn test_with_bound_multiple() {
        let result = bound! {
            with i = 1, j=2, k=3 => {
                i + j +k
            }
        };
        assert_eq!(result, 6);
        let a = B { field: 1 };
        let b = B { field: 2 };
        let result = bound! {
            with B{ field: i} = a, B{ field: j}= b =>{
                i+j
            }
        };
        assert_eq!(result, 3);
    }

    #[test]
    fn test_with_bound_multiple_and_expression() {
        let result = bound! {
            with i = 1, j=2, k=3 => i + j +k
        };
        assert_eq!(result, 6);
        let a = B { field: 1 };
        let b = B { field: 2 };
        let result = bound! {
            with B{ field: i} = a, B{ field: j} = b => i+j
        };
        assert_eq!(result, 3);
    }
}