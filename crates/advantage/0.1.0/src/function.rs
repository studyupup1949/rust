use super::*;

/// Abstraction of a function that can be examined by AD
pub trait Function: Send + Sync {
    /// Dimension of argument vector
    fn n(&self) -> usize;

    /// Dimension of result vector
    fn m(&self) -> usize;

    /// Evaluate the function
    fn eval(&self, x: DVector<ADouble>) -> DVector<ADouble>;

    /// Evaluate the function with floating point values
    fn eval_float(&self, x: DVector<f64>) -> DVector<f64> {
        let x = x.map(|e| ADouble::new(e, 0.0));
        self.eval(x).map(|e| e.value())
    }

    /// Inner chain structure if it exists
    fn chain(&self) -> Option<&FunctionChain> {
        None
    }

    /// Create a tape of the function
    fn tape(&self, x: &DVector<f64>) -> Box<dyn Tape> {
        let mut ctx = AContext::new();
        let mut input = x.map(|x| x.into());
        for x in input.iter_mut() {
            ctx.set_indep(x);
        }
        let output = self.eval(input);
        for y in output.iter() {
            ctx.set_dep(y);
        }
        Box::new(ctx.tape())
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct SimpleFunction<F>
where
    F: Fn(DVector<ADouble>) -> DVector<ADouble> + Send + Sync + 'static,
{
    n: usize,
    m: usize,
    f: F,
}

impl<F> SimpleFunction<F>
where
    F: Fn(DVector<ADouble>) -> DVector<ADouble> + Send + Sync + 'static,
{
    pub fn new(n: usize, m: usize, f: F) -> Self {
        Self { n, m, f }
    }
}

impl<F> Function for SimpleFunction<F>
where
    F: Fn(DVector<ADouble>) -> DVector<ADouble> + Send + Sync + 'static,
{
    fn n(&self) -> usize {
        self.n
    }

    fn m(&self) -> usize {
        self.m
    }

    fn eval(&self, x: DVector<ADouble>) -> DVector<ADouble> {
        (self.f)(x)
    }
}

/// A chain of function where each node takes its input from its predecessor
pub struct FunctionChain {
    funcs: Vec<Box<dyn Function>>,
}

#[allow(clippy::len_without_is_empty)]
impl FunctionChain {
    pub fn from_boxed(f: Box<dyn Function>) -> Self {
        let mut this = Self { funcs: Vec::new() };
        this.funcs.push(f);
        this
    }

    pub fn new<F: Function + 'static>(f: F) -> Self {
        Self::from_boxed(Box::new(f))
    }

    pub fn append<F: Function + 'static>(&mut self, f: F) {
        self.append_boxed(Box::new(f));
    }

    pub fn append_boxed(&mut self, f: Box<dyn Function>) {
        assert_eq!(self.funcs.last().unwrap().m(), f.n());
        self.funcs.push(f);
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = &dyn Function> {
        self.funcs.iter().flat_map(|f| {
            let iter: Box<dyn std::iter::Iterator<Item = &dyn Function>> = match f.chain() {
                Some(chain) => Box::new(chain.iter()),
                None => Box::new(std::iter::once(&**f)),
            };
            iter
        })
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn nth(&self, idx: usize) -> &dyn Function {
        self.iter().nth(idx).unwrap()
    }
}

impl Function for FunctionChain {
    fn n(&self) -> usize {
        self.funcs.first().unwrap().n()
    }

    fn m(&self) -> usize {
        self.funcs.last().unwrap().m()
    }

    fn chain(&self) -> Option<&FunctionChain> {
        Some(self)
    }

    fn eval(&self, x: DVector<ADouble>) -> DVector<ADouble> {
        let mut current = x;
        for func in self.funcs.iter() {
            current = func.eval(current);
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    adv_fn! {
        fn vector_10_max(x: [[10]]) -> [[1]] {
            let mut result = DVector::from_element(1, x[0]);
            for i in 1..x.nrows() {
                result[0] = result[0].max(x[i]);
            }
            result
        }
    }

    #[test]
    fn macro_function() {
        let x = DVector::from_vec((0..10).map(f64::from).collect());
        assert!((vector_10_max(x)[0] - 9.0).abs() < std::f64::EPSILON);

        let func_obj = __adv_vector_10_max();
        assert_eq!(func_obj.n(), 10);
        assert_eq!(func_obj.m(), 1);

        let tape = func_obj.tape(&DVector::zeros(10));
        assert_eq!(tape.num_indeps(), 10);
        assert_eq!(tape.num_deps(), 1);
        assert_eq!(tape.num_abs(), 9);
    }

    adv_fn! {
        fn ten_to_five(x: [[10]]) -> [[5]] {
            let mut result = DVector::<Scalar>::zeros(5);
            for i in 0..5 {
                result[i] = x[2*i] + x[2*i+1];
            }
            result
        }
    }

    adv_fn! {
        fn five_to_two(x: [[5]]) -> [[2]] {
            let mut result = DVector::<Scalar>::zeros(2);
            result[0] = x[0] + x[1] + x[2];
            result[1] = x[3] + x[4];
            result
        }
    }

    adv_fn! {
        fn two_to_one(x: [[2]]) -> [[1]] {
            let mut result = DVector::<Scalar>::zeros(1);
            result[0] = x[0] + x[1];
            result
        }
    }

    #[test]
    fn function_chain_eval() {
        let mut chain = FunctionChain::new(adv_fn_obj!(ten_to_five));
        chain.append(adv_fn_obj!(five_to_two));
        chain.append(adv_fn_obj!(two_to_one));

        assert_eq!(chain.len(), 3);
        assert_eq!(chain.n(), 10);
        assert_eq!(chain.m(), 1);

        let x = DVector::from_element(10, 1.0).map(ADouble::from);
        let y = chain.eval(x);
        assert_eq!(y.len(), 1);
        assert!((y[0].value() - 10.0).abs() < std::f64::EPSILON);
    }

    #[test]
    fn function_chain_eval_float() {
        let mut chain = FunctionChain::new(adv_fn_obj!(ten_to_five));
        chain.append(adv_fn_obj!(five_to_two));
        chain.append(adv_fn_obj!(two_to_one));

        let x = DVector::from_element(10, 1.0);
        let y = chain.eval_float(x);
        assert_eq!(y.len(), 1);
        assert!((y[0] - 10.0).abs() < std::f64::EPSILON);
    }

    adv_fn! {
        fn multiplicate(input: [[10]], factor: f64) -> [[10]] {
            let mut output = input;
            for i in 0..10 {
                output[i] *= factor;
            }
            output
        }
    }

    #[test]
    fn function_extra_args() {
        let obj = adv_fn_obj!(multiplicate, 10.0);
        let x = DVector::from_element(10, 1.0).map(ADouble::from);
        let y = obj.eval(x).map(|x| x.value());
        assert_eq!(y, DVector::from_element(10, 10.0));
    }

    adv_fn! {
        fn duplicate(input: [[1]], times: usize) -> [[times]] {
            DVector::from_element(times, input[0])
        }
    }

    #[test]
    fn function_dim_expressions() {
        let obj = adv_fn_obj!(duplicate, 10);
        let x = DVector::<f64>::zeros(1).map(ADouble::from);
        let y = obj.eval(x).map(|x| x.value());
        assert_eq!(y.nrows(), 10);
    }

    adv_fn! {
        fn duplicate2(input: [[1]], times1: usize, times2: usize) -> [[times1 * times2]] {
            DVector::from_element(times1 * times2, input[0])
        }
    }

    #[test]
    fn function_complex_dim_expressions() {
        let obj = adv_fn_obj!(duplicate2, 2, 5);
        let x = DVector::<f64>::zeros(1).map(ADouble::from);
        let y = obj.eval(x).map(|x| x.value());
        assert_eq!(y.nrows(), 10);
    }
}
