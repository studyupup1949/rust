//! # activation functions
//! 
//! `activation_functions` is a collection of functions, which can be used as
//! activation function for machine learning

/// containing the activation functions which need an f32 parameter and return a f32
pub mod f32 {
    /// calculate the `sigmoid` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = sigmoid(x);
    /// 
    /// println!("sigmoid({}) => {}",x,answer);
    /// ```
    pub fn sigmoid(x:f32) -> f32 {
        1 as f32 / (1 as f32 + std::f32::consts::E.powf(-x))
    }
    /// calculate the `binary step` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = bstep(x);
    /// 
    /// println!("bstep({}) => {}",x,answer);
    /// ```
    pub fn bstep(x:f32) -> f32 {
        if x<0 as f32 {
            0 as f32
        } else {
            1 as f32
        }
    }
    /// calculate the `tanh` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = tanh(x);
    /// 
    /// println!("tanh({}) => {}",x,answer);
    /// ```
    pub fn tanh(x:f32) -> f32 {
        (std::f32::consts::E.powf(x) - std::f32::consts::E.powf(-x)) / (std::f32::consts::E.powf(x) + std::f32::consts::E.powf(-x))
    }
    /// calculate the `rectified linear unit` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = relu(x);
    /// 
    /// println!("relu({}) => {}",x,answer);
    /// ```
    pub fn relu(x:f32) -> f32 {
        if x<=0 as f32 {
            0 as f32
        } else {
            1 as f32
        }
    }
    /// calculate the `sigmoid linear unit` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = silu(x);
    /// 
    /// println!("silu({}) => {}",x,answer);
    /// ```
    pub fn silu(x:f32) -> f32 {
        x / (1 as f32 + std::f32::consts::E.powf(-x))
    }
    /// calculate the `gaussian` to the given f32 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f32 = 0.5;
    /// let answer:f32 = gaussian(x);
    /// 
    /// println!("gaussian({}) => {}",x,answer);
    /// ```
    pub fn gaussian(x:f32) -> f32 {
        std::f32::consts::E.powf(-(x*x))
    }
}

/// containing the activation functions which need an f64 parameter and return a f64
pub mod f64 {
    /// calculate the `sigmoid` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = sigmoid(x);
    /// 
    /// println!("sigmoid({}) => {}",x,answer);
    /// ```
    pub fn sigmoid(x:f64) -> f64 {
        1 as f64 / (1 as f64 + std::f64::consts::E.powf(-x))
    }
    /// calculate the `binary step` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = bstep(x);
    /// 
    /// println!("bstep({}) => {}",x,answer);
    /// ```
    pub fn bstep(x:f64) -> f64 {
        if x<0 as f64 {            0 as f64        } else {            1 as f64        }
    }
    /// calculate the `tanh` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = tanh(x);
    /// 
    /// println!("tanh({}) => {}",x,answer);
    /// ```
    pub fn tanh(x:f64) -> f64 {
        (std::f64::consts::E.powf(x) - std::f64::consts::E.powf(-x)) / (std::f64::consts::E.powf(x) + std::f64::consts::E.powf(-x))
    }
    /// calculate the `rectified linear unit` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = relu(x);
    /// 
    /// println!("relu({}) => {}",x,answer);
    /// ```
    pub fn relu(x:f64) -> f64 {
        if x<=0 as f64 {
            0 as f64
        } else {
            1 as f64
        }
    }
    /// calculate the `sigmoid linear unit` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = silu(x);
    /// 
    /// println!("silu({}) => {}",x,answer);
    /// ```
    pub fn silu(x:f64) -> f64 {
        x / (1 as f64 + std::f64::consts::E.powf(-x))
    }
    /// calculate the `gaussian` to the given f64 number
    /// 
    /// # Examples
    /// 
    /// ```
    /// let x:f64 = 0.5;
    /// let answer:f64 = gaussian(x);
    /// 
    /// println!("gaussian({}) => {}",x,answer);
    /// ```
    pub fn gaussian(x:f64) -> f64 {
        std::f64::consts::E.powf(-(x*x))
    }
}