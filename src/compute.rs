/// takes all pendulum variables and returns their next increment
/*
pub fn step(th1: f64,  // angle of inner pendulum
        th2: f64,  // angle of outer pendulum
        w1: f64,    // angular velocity of inner pendulum
        w2: f64,    // angular velocity of outer pendulum
        m1: f64,    // mass of inner pendulum
        m2: f64,    // mass of outer pendulum
        L1: f64,    // length of inner pendulum
        L2: f64,    // length of outer pendulum
        g: f64      // gravitational acceleration
    ) -> (f64, f64) /* dw1, dw2 */ {

    // pre-compute reusable values (for optimization)
    let gamma = th1 - th2;
    let M = m1 + m2;
    let w1_2 = w1*w1;
    let w2_2 = w2*w2;
    let _2_sin_gamma = 2.0*gamma.sin();
    let cos_gamma = gamma.cos();

    // actual calculations
    let aux_0 = 2.0*m1 + m2 - m2*(2.0*gamma).sin();
    let aux_1 = -g*(2.0*m1 + m2)*th1.sin() - m2*g*(th1 - 2.0*th2).sin();
    let aux_2 = _2_sin_gamma*m2*(w2_2*L1 + w1_2*L1*cos_gamma);
    let aux_3 = _2_sin_gamma*(w1_2*L1*M + g*M*th1.cos() + w2_2*L2*m2*cos_gamma);
    let dw1 = (aux_1 - aux_2)/(L1*aux_0);
    let dw2 = aux_3/(L2*aux_0);

    return (dw1, dw2);
}
*/

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