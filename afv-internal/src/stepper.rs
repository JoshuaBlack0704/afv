use arduino_hal::{clock::MHz16, hal::usart::Usart0};
use embedded_hal::digital::v2::OutputPin;
pub enum StepperOpsError {
    AngleLimit,
}

pub trait StepperOps {
    /// Should get the current angle
    fn current_step(&self) -> i32;
    /// Will turn the stepper to the step index
    fn to_step(
        &mut self,
        step: i32,
        exit_on_max_steps: bool,
        serial: &mut Usart0<MHz16>,
    ) -> Result<i32, StepperOpsError>;
    fn home(&mut self, home_step: i32, serial: &mut Usart0<MHz16>);
}

pub struct StepperMotor<StepPin: OutputPin, DirPin: OutputPin> {
    max_clockwise: i32,
    min_clockwise: i32,
    current_step: i32,
    step: StepPin,
    dir: DirPin,
    microsteps: Option<u32>,
    microstep_time: u32,
    step_time_us: u32,
    inverted: bool,
}
pub fn convert_steps_angle(step: i32, steps_rev: u32) -> f32 {
    let angle_steps = steps_rev as f32 / 360.0;
    let angle = angle_steps * step as f32;
    angle
}
pub fn convert_angle_steps(angle: f32, steps_rev: u32) -> i32 {
    let steps_angle = 360.0 / steps_rev as f32;
    let steps = angle * steps_angle;
    steps as i32
}

impl<S: OutputPin, D: OutputPin> StepperMotor<S, D> {
    pub fn new(
        step_pin: S,
        dir_pin: D,
        max_clockwise: i32,
        min_clockwise: i32,
        microsteps: Option<u32>,
        step_time_us: u32,
        microstep_time: u32,
        inverted: bool,
    ) -> Self {
        Self {
            step: step_pin,
            dir: dir_pin,
            microsteps,
            current_step: 0,
            step_time_us,
            max_clockwise,
            min_clockwise,
            inverted,
            microstep_time,
        }
    }
}

impl<S: OutputPin, D: OutputPin> StepperOps for StepperMotor<S, D> {
    fn current_step(&self) -> i32 {
        self.current_step
    }

    fn to_step(
        &mut self,
        step: i32,
        exit_on_step_limit: bool,
        serial: &mut Usart0<MHz16>,
    ) -> Result<i32, StepperOpsError> {
        let _ = ufmt::uwriteln!(
            serial,
            "Stepper going to step {} from step {}",
            step,
            self.current_step()
        );
        let steps = step - self.current_step();

        self.current_step += steps;

        if self.current_step > self.max_clockwise {
            self.current_step = self.max_clockwise;
            if exit_on_step_limit {
                return Err(StepperOpsError::AngleLimit);
            }
        }
        if self.current_step < self.min_clockwise {
            self.current_step = self.min_clockwise;
            if exit_on_step_limit {
                return Err(StepperOpsError::AngleLimit);
            }
        }

        if steps.is_positive() {
            if !self.inverted {
                let _ = self.dir.set_high();
            } else {
                let _ = self.dir.set_low();
            }
        } else {
            if !self.inverted {
                let _ = self.dir.set_low();
            } else {
                let _ = self.dir.set_high();
            }
        }

        for _ in 0..steps.abs() {
            arduino_hal::delay_us(self.step_time_us);
            let mut microsteps = 1;
            if let Some(steps) = self.microsteps {
                microsteps = steps;
            }

            for _ in 0..microsteps {
                let _ = self.step.set_high();
                arduino_hal::delay_us(self.microstep_time);
                let _ = self.step.set_low();
                arduino_hal::delay_us(self.microstep_time);
            }
        }

        Ok(self.current_step())
    }

    fn home(&mut self, home_step: i32, serial: &mut Usart0<MHz16>) {
        let _ = self.to_step(home_step, false, serial);
        let _ = self.to_step(0, false, serial);
    }
}
