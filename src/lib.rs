#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

use std::collections::VecDeque;
use std::sync::Arc;

use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use rand_xoshiro::rand_core::RngCore;

mod compute;

struct Effect {
    // Store a handle to the plugin's parameter object.
    params: Arc<EffectParameters>,

    // meta variables
    sr: f32,
    rng: Xoshiro256Plus,
    scale: f64,     // scaling factor for sr independence of integrals

    // pendulum variables
    th1: f64,
    th2: f64,
    osc1_th: f64,
    osc2_th: f64,
    w1: f64,
    w2: f64,
}

struct EffectParameters {
    // Pendulum parameters
    len_ratio: AtomicFloat,     // len of L1 vs len of L2 range 0 to 1
    scale: AtomicFloat,         // scale the physics of the pendulum, affects
                                // the mean frequency of the pendulum

    // Driving osc parameters
    o1_amt: AtomicFloat,        // amount of oscillator 1 driving the first pendulum
    o2_amt: AtomicFloat,        // amount of oscillator 2 driving the second pendulum
    o1_f: AtomicFloat,          // frequency of oscillator 1 in octaves
    o2_f: AtomicFloat,          // frequency of oscillator 2 in octaves
    o1_fine: AtomicFloat,       // fine-tune of osc 1, ±1 octave range
    o2_fine: AtomicFloat,       // fine-tune of osc 2, ±1 octave range
    o2_to_o1_mod: AtomicFloat,  // FM osc2 into osc1 0-200% range
    o1_to_o2_mod: AtomicFloat,  // FM osc1 into osc2 0-200% range
}

impl Default for Effect {
    fn default() -> Effect {
        Effect {
            params: Arc::new(EffectParameters::default()),

            // meta variables
            sr: 44100.0,
            rng: Xoshiro256Plus::seed_from_u64(69_420),
            scale: 1.0,

            // pendulum variables
            th1: 3.0,
            th2: 4.0,
            osc1_th: 0.0,
            osc2_th: 0.0,
            w1: 0.0,
            w2: 0.0,
        }
    }
}

impl Default for EffectParameters {
    fn default() -> EffectParameters {
        EffectParameters {
            // Pendulum parameters
            len_ratio: AtomicFloat::new(0.5),
            scale: AtomicFloat::new(0.5),

            // Driving osc parameters
            o1_amt: AtomicFloat::new(0.0),
            o2_amt: AtomicFloat::new(0.0),
            o1_f: AtomicFloat::new(0.5),
            o2_f: AtomicFloat::new(0.5),
            o1_fine: AtomicFloat::new(0.5),
            o2_fine: AtomicFloat::new(0.5),
            o2_to_o1_mod: AtomicFloat::new(0.0),
            o1_to_o2_mod: AtomicFloat::new(0.0),
        }
    }
}

// All plugins using `vst` also need to implement the `Plugin` trait.  Here, we
// define functions that give necessary info to our host.
// TODO: make it accept and consume MIDI (and not use it) so that Ableton doesn't
// categorize it as an effect.
impl Plugin for Effect {
    fn get_info(&self) -> Info {
        Info {
            name: "CHAOS_OSC".to_string(),
            vendor: "Flux-Audio".to_string(),
            unique_id: 40942320,
            version: 020,
            inputs: 0,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 10,
            category: Category::Generator,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32){
        self.sr = rate;
        self.scale = 44100.0 / rate as f64; 
    }

    // called once
    fn init(&mut self) {}

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, outputs) = buffer.split();

        // Iterate over outputs as (&mut f32, &mut f32)
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());

        // process
        for (left_out, right_out) in stereo_out{
            // get params
            let o1_amt = self.params.o1_amt.get() as f64;
            let o2_amt = self.params.o2_amt.get() as f64;
            let o1_f = (self.params.o1_f.get()*8.0 + self.params.o1_fine.get()) as f64;
            let o2_f = (self.params.o2_f.get()*8.0 + self.params.o2_fine.get()) as f64;
            let o2_to_o1_mod = self.params.o2_to_o1_mod.get() as f64;
            let o1_to_o2_mod = self.params.o1_to_o2_mod.get() as f64;
            let mut scale = self.params.scale.get() as f64;
            scale = scale*scale*8.0;
            /*
            let scale = self.params.scale.get()/105.0;
            let m1 = scale;
            let m2 = scale;
            
            let g = scale;
            */
            let l2 = (self.params.len_ratio.get()*2.0 + 0.01) as f64;
            let l1 = 2.03 - l2;
            
            // oscillators
            let osc1 = compute::oct_to_rad(o1_f + self.osc2_th.sin()*o2_to_o1_mod, self.sr);
            let osc2 = compute::oct_to_rad(o2_f + self.osc1_th.sin()*o1_to_o2_mod, self.sr);
            self.osc1_th = compute::wrap(self.osc1_th + osc1);
            self.osc2_th = compute::wrap(self.osc2_th + osc2);

            // solve dif.e.
            let (dth1, dth2, dw1, dw2) = compute::step(
                self.th1, self.th2,
                self.w1, self.w2,
                l1, l2
            );
            
            // update state
            // note that the angular velocity and it's rate of change are both
            // limited at 1.0 with a saturator, this is to avoid exploding
            // floating point errors resulting into NaN values. While more
            // robust ways of doing this might exist, this is how it was done
            // in the original Reaktor module, and it is now part of its sound

            /*
            let sat_amt = 0.453515/(scale*self.scale) + 20.0;
            let dw1_sat = (dw1/sat_amt).tanh()*sat_amt;
            let dw2_sat = (dw2/sat_amt).tanh()*sat_amt;
            
            */

            //self.w1 = ((self.w1 + dw1_sat /*+ nse1*/)*self.scale).tanh();
            //self.w2 = ((self.w2 + dw2_sat /*+ nse2*/)*self.scale).tanh();
            //self.w1 = ((self.w1 + dw1)*self.scale).tanh();
            //self.w2 = ((self.w2 + dw2)*self.scale).tanh();
            //let dth1 = compute::fade(self.w1, o1_amt, osc1);
            //let dth2 = compute::fade(self.w2, o2_amt, osc2);
            let sat = 15.0/(scale + 0.01);
            self.w1 = ((self.w1 + dw1*0.1*self.scale*scale)/sat).tanh()*sat;
            self.w2 = ((self.w2 + dw2*0.1*self.scale*scale)/sat).tanh()*sat;
            let dth1 = compute::fade((dth1*0.1*self.scale*scale/sat).tanh()*sat, o1_amt, osc1);
            let dth2 = compute::fade((dth2*0.1*self.scale*scale/sat).tanh()*sat, o2_amt, osc2);
            self.th1 = compute::wrap(self.th1 + dth1);
            self.th2 = compute::wrap(self.th2 + dth2);

            *left_out = self.th1.sin() as f32;
            *right_out = self.th2.sin() as f32;
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for EffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            // Pendulum parameters
            0 => self.len_ratio.get(),
            1 => self.scale.get(),

            // Driving osc parameters
            2 => self.o1_amt.get(),
            3 => self.o2_amt.get(),
            4 => self.o1_f.get(),
            5 => self.o2_f.get(),
            6 => self.o1_fine.get(),
            7 => self.o2_fine.get(),
            8 => self.o2_to_o1_mod.get(),
            9 => self.o1_to_o2_mod.get(),
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            // Pendulum parameters
            0 => self.len_ratio.set(val),
            1 => self.scale.set(val),

            // Driving osc parameters
            2 => self.o1_amt.set(val),
            3 => self.o2_amt.set(val),
            4 => self.o1_f.set(val),
            5 => self.o2_f.set(val),
            6 => self.o1_fine.set(val),
            7 => self.o2_fine.set(val),
            8 => self.o2_to_o1_mod.set(val),
            9 => self.o1_to_o2_mod.set(val),
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            // Pendulum parameters
            0 => format!("L1: {:.2}, L2: {:.2}", 1.0-self.len_ratio.get(), self.len_ratio.get()),
            1 => format!("{:.2}", self.scale.get()),

            // Driving osc parameters
            2 => format!("{:.2}", self.o1_amt.get()),
            3 => format!("{:.2}", self.o2_amt.get()),
            4 => format!("{:.2}", self.o1_f.get()*8.0),
            5 => format!("{:.2}", self.o2_f.get()*8.0),
            6 => format!("{:.2}", self.o1_fine.get()),
            7 => format!("{:.2}", self.o2_fine.get()),
            8 => format!("{:.1}", self.o2_to_o1_mod.get()*200.0),
            9 => format!("{:.1}", self.o1_to_o2_mod.get()*200.0),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "L1 <=> L2",
            1 => "- <=> +",
            2 => "O1",
            3 => "O2",
            4 => "F1",
            5 => "F2",
            6 => "F1.f",
            7 => "F2.f",
            8 => "M1",
            9 => "M2",
            _ => "",
        }
        .to_string()
    }
}

// This part is important!  Without it, our plugin won't work.
plugin_main!(Effect);