use arduino_hal::{hal::usart::Usart0, clock::MHz16};
use embedded_hal::digital::v2::OutputPin;
pub enum StepperOpsError{
    AngleLimit,
}

pub trait StepperOps{
    /// Should get the current angle
    fn current_angle(&self) -> f32;
    fn current_step(&self) -> i32;
    fn max_clockwise_step(&self) -> Option<u32>;
    fn max_counter_clockwise_step(&self) -> Option<u32>;
    /// The number and direction of steps
    /// Positive steps is clockwise, negative for counter_clockwise
    fn step(&mut self, steps: i32, ignore_max_steps: bool, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError>;
    fn micro_step(&mut self, steps: i32, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError>;
    /// Will turn the stepper to the desired angle
    fn to_angle(&mut self, angle: f32, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError>;
    /// Will turn the stepper to the step index
    fn to_step(&mut self, step: i32, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError>;
}

pub struct StepperMotor<StepPin: OutputPin, DirPin: OutputPin>{
    max_clockwise_step: Option<u32>,
    max_counter_clockwise_step: Option<u32>,
    current_step: i32,
    step: StepPin,
    dir: DirPin,
    steps_rev: u32,
    microsteps: Option<u32>,
    step_time_us: u32,
    inverted: bool,
}
pub fn convert_steps_angle(step: i32, steps_rev: u32) -> f32{
    let angle_steps = steps_rev as f32 / 360.0;
    let angle = angle_steps * step as f32;
    angle
}
pub fn convert_angle_steps(angle: f32, steps_rev: u32) -> i32{
    let steps_angle = 360.0 / steps_rev as f32;
    let steps = angle * steps_angle;
    steps as i32
}

impl<S: OutputPin, D: OutputPin> StepperMotor<S,D>{
    pub fn new(step_pin: S, dir_pin: D, max_clockwise_step: Option<u32>, max_counter_clockwise_step: Option<u32>, steps_rev: u32, microsteps: Option<u32>, step_time_us: u32, inverted: bool) -> Self{
        Self{
            step: step_pin,
            dir: dir_pin,
            steps_rev,
            microsteps,
            inverted,
            max_clockwise_step,
            max_counter_clockwise_step,
            current_step: 0,
            step_time_us,
        }
    }
}

impl<S: OutputPin, D: OutputPin> StepperOps for StepperMotor<S,D>{
    fn current_angle(&self) -> f32 {
        let angle_step = 360.0 / self.steps_rev as f32;

        self.current_step() as f32 * angle_step
    }

    fn current_step(&self) -> i32 {
        self.current_step
    }

    fn max_clockwise_step(&self) -> Option<u32> {
        self.max_clockwise_step
    }

    fn max_counter_clockwise_step(&self) -> Option<u32> {
        self.max_counter_clockwise_step
    }

    fn step(&mut self, steps: i32, ignore_max_steps: bool, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError> {
        let mut steps = steps;
        if self.inverted{
            steps = -steps;
        }
        let _ = ufmt::uwriteln!(serial, "Step command {}", steps);
        if steps.is_positive(){
            let _ = self.dir.set_low();
        }
        else{
            let _ = self.dir.set_high();
        }

        self.current_step += steps;
        let _ = ufmt::uwriteln!(serial, "New current step {}", self.current_step);


        if let Some(max) = self.max_clockwise_step{
            if self.current_step.is_positive() && self.current_step as u32 > max{
                self.current_step = max as i32;
                if !ignore_max_steps{
                    return Err(StepperOpsError::AngleLimit);
                }
            }
        }
        if let Some(max) = self.max_counter_clockwise_step{
            if self.current_step.is_negative() && self.current_step as u32 > max{
                self.current_step = -(max as i32);
                if !ignore_max_steps{
                    return Err(StepperOpsError::AngleLimit);
                }
            }
        }
        
        for _ in 0..200{
            arduino_hal::delay_us(self.step_time_us);
            let mut microsteps = 1;
            if let Some(steps) = self.microsteps{
                microsteps = steps;
            }

            for _ in 0..microsteps{
                let _ = self.step.set_high();
                arduino_hal::delay_us(50);
                let _ = self.step.set_low();
                arduino_hal::delay_us(50);
            }
        }

        Ok(self.current_step)
    }

    fn micro_step(&mut self, _steps: i32, _serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError> {
        unimplemented!()
    }

    fn to_angle(&mut self, angle: f32, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError> {
        let current_angle = self.current_angle();
        let angle_step = 360.0 / self.steps_rev as f32;
        let angle = angle - current_angle;

        let steps = angle_step * angle;

        self.step(steps as i32, false, serial)
    }

    fn to_step(&mut self, step: i32, serial: &mut Usart0<MHz16>) -> Result<i32, StepperOpsError> {
        let mut steps = step - self.current_step();
        if self.inverted{
            steps = -steps;
        }
        let _ = ufmt::uwriteln!(serial, "To step to {}, from {} to {}", steps, self.current_step(), step);
        self.step(steps, true, serial)
    }
}