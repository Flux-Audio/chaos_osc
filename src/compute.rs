/// this is the simplified version of the double pendulum differential equation
/// system, the old one was broken
pub fn step(th1: f64, th2: f64, w1: f64, w2: f64, l1: f64, l2: f64)
        -> (f64, f64, f64, f64){

    let c = (th1 - th2).cos();
    let s = (th1 - th2).sin(); 

    let dth1 = w1;
    let dth2 = w2;

    let dw1 = (th2.sin()*c - s*(l1*w1*w1*c + l2*w2*w2) - th1.sin()) /
            l1 / (1.00 + s*s);
    let dw2 = ((l1*w1*w1*s - th2.sin() + th1.sin()*c) + l2*w2*w2*s*c) /
            l2 / (1.00 + s*s);
    return (dth1, dth2, dw1, dw2);
}

/// wraps an angle from -pi/2 to +pi/2
pub fn wrap(theta: f64) -> f64{
    return theta.rem_euclid(std::f64::consts::PI*2.0);
}

/// crossfades two values, x should be in the interval [0.0, 1.0]
pub fn fade(_0: f64, x: f64, _1: f64) -> f64{ return _0*(1.0 - x) + _1*x; }

/// takes 1v/oct and turns it into rad/T
pub fn oct_to_rad(oct: f64, sr: f32) -> f64{
    let f = oct.exp2()*17.3238;     // octave to frequency, centered around
                                    // middle C#
    return f/(sr as f64)*6.28318530718;      // frequency to radians per sample
}