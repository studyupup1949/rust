use std::f32;
use std::f64;

pub fn logistic32(x: f32, c: f32) -> f32
{
	1.0 / (1.0 + (-c*x).exp())
}

pub fn logistic64(x: f64, c: f64) -> f64
{
	1.0 / (1.0 + (-c*x).exp())
}

pub fn tanh32(x: f32, c: f32) -> f32
{
	(x*c).tanh()
}

pub fn tanh64(x: f64, c: f64) -> f64
{
	(x*c).tanh()
}

pub fn fast32(x: f32, c: f32) -> f32
{
	(x*c)/(1.0+(x*c).abs())
}

pub fn fast64(x: f64, c: f64) -> f64
{
	(x*c)/(1.0+(x*c).abs())
}

pub fn softplus32(x: f32, c: f32) -> f32
{
	(1.0+(x*c).exp()).ln()
}

pub fn softplus64(x: f64, c: f64) -> f64
{
	(1.0+(x*c).exp()).ln()
}

pub fn rectifier32(x: f32, c: f32) -> f32
{
	(x*c).max(0.0)
}

pub fn rectifier64(x: f64, c: f64) -> f64
{
	(x*c).max(0.0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_logistic() 
    {
    	assert!(logistic64())
    }
}
