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

    sr: f32,
    rng: Xoshiro256Plus,
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

            sr: 44100.0,
            rng: Xoshiro256Plus::seed_from_u64(69_420),
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
    }

    // called once
    fn init(&mut self) {}

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, outputs) = buffer.split();

        // Iterate over inputs as (&f32, &f32)
        // let (l, r) = inputs.split_at(1);
        // let stereo_in = l[0].iter().zip(r[0].iter());

        // Iterate over outputs as (&mut f32, &mut f32)
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());

        // get all params TODO:
        /*
        let div = (self.params.div.get()*11.5 + 1.0) as usize;   // scale to int in 1..6
        */

        // process
        for (left_out, right_out) in stereo_out{
            *left_out = 0.0;
            *right_out = 0.0;
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