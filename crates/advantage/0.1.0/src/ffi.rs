#![allow(non_camel_case_types)]
use super::*;
use nalgebra::{DMatrix, DVector};
use num::Zero;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum adv_error {
    ADV_ERROR_SUCCESS,
    ADV_ERROR_DIM_MISMATCH,
}

type Result<T> = std::result::Result<T, adv_error>;

impl From<Result<()>> for adv_error {
    fn from(result: Result<()>) -> adv_error {
        match result {
            Ok(_) => adv_error::ADV_ERROR_SUCCESS,
            Err(err) => err,
        }
    }
}

impl std::ops::Try for adv_error {
    type Ok = ();
    type Error = adv_error;

    fn into_result(self) -> Result<()> {
        match self {
            adv_error::ADV_ERROR_SUCCESS => Ok(()),
            x @ _ => Err(x),
        }
    }

    fn from_error(v: adv_error) -> Self {
        v
    }

    fn from_ok(_: ()) -> Self {
        adv_error::ADV_ERROR_SUCCESS
    }
}

// `Context` bindings

#[no_mangle]
pub extern "C" fn adv_context_new() -> *mut AContext {
    Box::leak(Box::new(AContext::new()))
}

#[no_mangle]
pub unsafe extern "C" fn adv_context_free(this: *mut AContext) {
    Box::from_raw(this);
}

#[no_mangle]
pub extern "C" fn adv_context_new_independent(this: &'static mut AContext) -> *mut ADouble {
    Box::leak(Box::new(this.new_indep(0.0)))
}

#[no_mangle]
pub extern "C" fn adv_context_set_dependent(this: &mut AContext, val: &ADouble) {
    this.set_dep(val);
}

// `ADouble` bindings

#[no_mangle]
pub unsafe extern "C" fn adv_double_default() -> *mut ADouble {
    Box::leak(Box::new(ADouble::zero()))
}

#[no_mangle]
pub unsafe extern "C" fn adv_double_from_value(val: f64) -> *mut ADouble {
    Box::leak(Box::new(ADouble::from(val)))
}

#[no_mangle]
pub unsafe extern "C" fn adv_double_free(this: *mut ADouble) {
    Box::from_raw(this);
}

#[no_mangle]
pub extern "C" fn adv_double_copy(this: &ADouble) -> *mut ADouble {
    Box::leak(Box::new(this.clone()))
}

// `drivers` bindings

#[repr(C)]
pub struct adv_vector {
    pub size: usize,
    pub data: *mut f64,
}

#[repr(C)]
pub struct adv_const_vector {
    pub size: usize,
    pub data: *const f64,
}

impl adv_vector {
    pub unsafe fn copy_from_dvec(&mut self, dvec: &DVector<f64>) -> Result<()> {
        if self.size == dvec.nrows() {
            for idx in 0..self.size {
                *self.data.add(idx) = dvec[idx];
            }
            Ok(())
        } else {
            Err(adv_error::ADV_ERROR_DIM_MISMATCH)
        }
    }
}

impl adv_const_vector {
    pub unsafe fn to_dvec(&self) -> DVector<f64> {
        let mut dvec = DVector::from_element(self.size, 0.0);
        for idx in 0..self.size {
            dvec[idx] = *self.data.add(idx);
        }
        dvec
    }
}

#[repr(C)]
pub struct adv_matrix {
    pub rows: usize,
    pub cols: usize,
    pub pitch: usize,
    pub data: *mut f64,
}

#[repr(C)]
pub struct adv_const_matrix {
    pub rows: usize,
    pub cols: usize,
    pub pitch: usize,
    pub data: *const f64,
}

impl adv_matrix {
    pub unsafe fn copy_from_dmat(&mut self, dmat: &DMatrix<f64>) -> Result<()> {
        if self.rows == dmat.nrows() && self.cols == dmat.ncols() {
            for i in 0..self.rows {
                for j in 0..self.cols {
                    *self.data.add(i * self.pitch + j) = dmat[(i, j)];
                }
            }
            Ok(()).into()
        } else {
            Err(adv_error::ADV_ERROR_DIM_MISMATCH)
        }
    }
}

impl adv_const_matrix {
    pub unsafe fn to_dmat(&self) -> DMatrix<f64> {
        let mut dmat = DMatrix::from_element(self.rows, self.cols, 0.0);
        for i in 0..self.rows {
            for j in 0..self.cols {
                dmat[(i, j)] = *self.data.add(i * self.pitch + j);
            }
        }
        dmat
    }
}

// `ADouble` operation bindings

macro_rules! binary_operation {
    ($op_name:ident, $op:tt) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<adv_op_ $op_name>](a: &'static ADouble, b: &'static ADouble, result: &'static mut *mut ADouble) {
                *result = Box::leak(Box::new(*a $op *b));
            }
        }
    }
}

binary_operation!(add, +);
binary_operation!(sub, -);
binary_operation!(mul, *);
binary_operation!(div, /);

macro_rules! unary_function {
    ($func_name:ident) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<adv_ $func_name>](x: &'static ADouble, result: &'static mut *mut ADouble) {
                *result = Box::leak(Box::new(x.$func_name()));
            }
        }
    }
}

unary_function!(sin);
unary_function!(cos);
unary_function!(tan);
unary_function!(abs);
unary_function!(exp);
unary_function!(ln);

macro_rules! binary_function {
    ($func_name:ident) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<adv_ $func_name>](a: &'static ADouble, b: &'static ADouble, result: &'static mut *mut ADouble) {
                *result = Box::leak(Box::new(a.$func_name(*b)));
            }
        }
    }
}

binary_function!(min);
binary_function!(max);
