use attributes::AttrIdentFilter;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

/// Library-specific modules
mod attributes;
mod traits;
mod arg;

/// implementation argument structure for implementing Add between Vector types
mod impl_vector_add;
use crate::impl_vector_add::ImplVectorAdd;

/// implementation argument structure for implementing Mul between Vector types
mod impl_dot_prod;
use crate::impl_dot_prod::ImplDotProduct;

mod impl_vector_sub;
use crate::impl_vector_sub::ImplVectorSub;

fn mutability(input: &AttrIdentFilter) -> (bool, bool) {
    if input.match_against("mut_both", true) {
        (true, true)
    } else {
        (
            input.match_against("mut_left", true),
            input.match_against("mut_right", true)
        )
    }
}

/// Highly efficient way to implement immutable addition between two vector types.
///
/// This is only intended to be used within the adv_linalg_library, and not for
/// public use.
///
/// ## Example
/// ```
/// impl_vector_add!(
///     impl<T: Clone + Add<Output = T>> Add for [Vector<T>, MutVector<T>] + [Vector<T> + MutVector<T>];
///     impl<'lhs, 'rhs, T: Clone + Add<Output = T>> Add for [VectorSlice<'lhs, T>] + [Vector<T> + VectorSlice<'rhs, T>];
///     #[mut_left] impl<'rhs, T: Clone + Add<Output = T>> Add for [MutVector<T>] + [VectorSlice<'rhs, T>]
/// );
/// ```
/// 
/// ## Base Addition Implementations
/// 
/// The code will try to implement the Add trait given a `LeftType` and a
/// `RightType` in the following variations for each left-type, right-type pairs:
/// - LeftType + RightType
/// - &LeftType + RightType
/// - LeftType + &RightType
/// - &LeftType + &RightType
/// 
/// # Mutable Addition Implementations
/// To mark a line having a side that contain all mutable types, use
/// 1. `#[mut_left]`
/// 2. `#[mut_right]` or
/// 3. `#[mut_both]`
/// 
/// This will implement mutability specific implementations alongside the base implementations.
/// 
/// If the attribute `#[mut_left]` or `#[mut_both]` are used,
/// then this macro will automatically import the lifetime `'lhs` into the
/// implementation (only when neccessary).
///
/// ## Safety
/// To use this macro, the automatic implementation assumes the following:
/// - The LeftType and RightType both implement a `Fn(&self) -> usize` method
/// - The LeftType and RightType both implement Index
/// - If the LeftType is mutable, then it must implement IndexMut
/// - Passed generic annotations are used properly
#[proc_macro]
pub fn impl_vector_add(tokens: TokenStream1) -> TokenStream1 {
    // try to parse the macro input as what is expected
    let main = parse_macro_input!(tokens as ImplVectorAdd);

    let mut output = TokenStream2::new();
    for full_arg in main.args {
        let (left_is_mut, right_is_mut) = mutability(&full_arg.mutability);
        let no_std = full_arg.mut_only.match_against("no_std", true);

        let arg = full_arg.arg;

        // pull out global information from the main struct
        //
        // (These are used for each quote!{} macro call)
        let generics = arg.generics.clone();
        let mut_generics = arg.mut_generics();

        //The Add implmentations
        for lhs_ty in arg.lhs_tys {
            for rhs_ty in arg.rhs_tys.clone() {
                if !no_std {
                    output.extend(quote! {
                        impl #generics Add<#rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Add<&#rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Add<#rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Add<&#rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
                    });
                }
                if left_is_mut {
                    output.extend(quote!{
                        impl #mut_generics Add<#rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn add(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() + rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                        
                        impl #mut_generics Add<&#rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn add(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() + rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                    })
                }

                if right_is_mut && !no_std {
                    output.extend(quote!{
                        impl #generics Add<&mut #rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }

                        impl #generics Add<&mut #rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn add(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() + rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
                    })
                }

                if left_is_mut && right_is_mut {
                    output.extend(quote!{
                        impl #mut_generics Add<&mut #rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn add(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be added together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() + rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                    })
                }
            }
        }
    }

    TokenStream1::from(output)
}

#[proc_macro]
pub fn impl_dot_product(tokens: TokenStream1) -> TokenStream1 {
    // try to parse the macro input as what is expected
    let main = parse_macro_input!(tokens as ImplDotProduct);

    let mut output = TokenStream2::new();
    for full_arg in main.args {
        let (left_is_mut, right_is_mut) = mutability(&full_arg.mutability);
        let arg = full_arg.arg;
        
        // pull out global information from the main struct
        //
        // (These are used for each quote!{} macro call)
        let generics = arg.generics.clone();

        //The Mul implmentations
        for lhs_ty in arg.lhs_tys {
            for rhs_ty in arg.rhs_tys.clone() {
                output.extend(quote! {
                    impl #generics Mul<#rhs_ty> for #lhs_ty
                    {
                        type Output = T;

                        fn mul(self, rhs: #rhs_ty) -> Self::Output {
                            let length = self.len();

                            if (length) != rhs.len() {
                                panic!("Cannot find dot product of two differently sized vectors.")
                            }

                            let mut product = T::default();
                            for idx in 0..length {
                                product = product + self[idx].clone() * rhs[idx].clone()
                            }
                            product
                        }
                    }

                    impl #generics Mul<&#rhs_ty> for #lhs_ty
                    {
                        type Output = T;

                        fn mul(self, rhs: &#rhs_ty) -> Self::Output {
                            let length = self.len();

                            if (length) != rhs.len() {
                                panic!("Cannot find dot product of two differently sized vectors.")
                            }

                            let mut product = T::default();
                            for idx in 0..length {
                                product = product + self[idx].clone() * rhs[idx].clone()
                            }
                            product
                        }
                    }

                    impl #generics Mul<#rhs_ty> for &#lhs_ty
                    {
                        type Output = T;

                        fn mul(self, rhs: #rhs_ty) -> Self::Output {
                            let length = self.len();

                            if (length) != rhs.len() {
                                panic!("Cannot find dot product of two differently sized vectors.")
                            }

                            let mut product = T::default();
                            for idx in 0..length {
                                product = product + self[idx].clone() * rhs[idx].clone()
                            }
                            product
                        }
                    }

                    impl #generics Mul<&#rhs_ty> for &#lhs_ty
                    {
                        type Output = T;

                        fn mul(self, rhs: &#rhs_ty) -> Self::Output {
                            let length = self.len();

                            if (length) != rhs.len() {
                                panic!("Cannot find dot product of two differently sized vectors.")
                            }

                            let mut product = T::default();
                            for idx in 0..length {
                                product = product + self[idx].clone() * rhs[idx].clone()
                            }
                            product
                        }
                    }
                });

                if left_is_mut {
                    output.extend(quote!{
                        impl #generics Mul<#rhs_ty> for &mut #lhs_ty
                        {
                            type Output = T;
    
                            fn mul(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Cannot find dot product of two differently sized vectors.")
                                }
    
                                let mut product = T::default();
                                for idx in 0..length {
                                    product = product + self[idx].clone() * rhs[idx].clone()
                                }
                                product
                            }
                        }

                        impl #generics Mul<&#rhs_ty> for &mut #lhs_ty
                        {
                            type Output = T;
    
                            fn mul(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Cannot find dot product of two differently sized vectors.")
                                }
    
                                let mut product = T::default();
                                for idx in 0..length {
                                    product = product + self[idx].clone() * rhs[idx].clone()
                                }
                                product
                            }
                        }
                    })
                }

                if right_is_mut {
                    output.extend(quote!{
                        impl #generics Mul<&mut #rhs_ty> for #lhs_ty
                        {
                            type Output = T;
    
                            fn mul(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Cannot find dot product of two differently sized vectors.")
                                }
    
                                let mut product = T::default();
                                for idx in 0..length {
                                    product = product + self[idx].clone() * rhs[idx].clone()
                                }
                                product
                            }
                        }

                        impl #generics Mul<&mut #rhs_ty> for &#lhs_ty
                        {
                            type Output = T;
    
                            fn mul(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Cannot find dot product of two differently sized vectors.")
                                }
    
                                let mut product = T::default();
                                for idx in 0..length {
                                    product = product + self[idx].clone() * rhs[idx].clone()
                                }
                                product
                            }
                        }
                    })
                }

                if left_is_mut && right_is_mut {
                    output.extend(quote!{
                        impl #generics Mul<&mut #rhs_ty> for &mut #lhs_ty
                        {
                            type Output = T;
    
                            fn mul(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Cannot find dot product of two differently sized vectors.")
                                }
    
                                let mut product = T::default();
                                for idx in 0..length {
                                    product = product + self[idx].clone() * rhs[idx].clone()
                                }
                                product
                            }
                        }
                    })
                }
            }
        }
    }

    TokenStream1::from(output)
}

#[proc_macro]
pub fn impl_vector_sub(tokens: TokenStream1) -> TokenStream1 {
    // try to parse the macro input as what is expected
    let main = parse_macro_input!(tokens as ImplVectorSub);

    let mut output = TokenStream2::new();
    for full_arg in main.args {
        let (left_is_mut, right_is_mut) = mutability(&full_arg.mutability);
        let no_std = full_arg.mut_only.match_against("no_std", true);

        let arg = full_arg.arg;

        // pull out global information from the main struct
        //
        // (These are used for each quote!{} macro call)
        let generics = arg.generics.clone();
        let mut_generics = arg.mut_generics();

        //The Sub implmentations
        for lhs_ty in arg.lhs_tys {
            for rhs_ty in arg.rhs_tys.clone() {
                if !no_std {
                    output.extend(quote! {
                        impl #generics Sub<#rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Sub<&#rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Sub<#rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
    
                        impl #generics Sub<&#rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
                    });
                }
                if left_is_mut {
                    output.extend(quote!{
                        impl #mut_generics Sub<#rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn sub(self, rhs: #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() - rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                        
                        impl #mut_generics Sub<&#rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn sub(self, rhs: &#rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() - rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                    })
                }

                if right_is_mut && !no_std {
                    output.extend(quote!{
                        impl #generics Sub<&mut #rhs_ty> for #lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }

                        impl #generics Sub<&mut #rhs_ty> for &#lhs_ty
                        {
                            type Output = Vector<T>;
    
                            fn sub(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                let mut params = Vec::with_capacity(length);
                                for idx in 0..length {
                                    params.push(self[idx].clone() - rhs[idx].clone())
                                }
                                Vector::from(params)
                            }
                        }
                    })
                }

                if left_is_mut && right_is_mut {
                    output.extend(quote!{
                        impl #mut_generics Sub<&mut #rhs_ty> for &'lhs mut #lhs_ty
                        {
                            type Output = &'lhs mut #lhs_ty;
    
                            fn sub(self, rhs: &mut #rhs_ty) -> Self::Output {
                                let length = self.len();
    
                                if (length) != rhs.len() {
                                    panic!("Vectors with different sizes cannot be subtracted together.")
                                }
    
                                for idx in 0..length {
                                    self[idx] = self[idx].clone() - rhs[idx].clone()
                                }
    
                                self
                            }
                        }
                    })
                }
            }
        }
    }

    TokenStream1::from(output)
}