use arduino_hal::{
    clock::MHz16,
    hal::usart::Usart0,
    prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead},
    I2c,
};

use crate::lidar::I2cLidarOps;

pub const LIDAR_ADDRESS: u8 = 0x62;
pub struct ControlRegister(u8);
impl From<ControlRegister> for u8 {
    fn from(cr: ControlRegister) -> Self {
        cr.0
    }
}
impl ControlRegister {
    pub const ACQ_COMMAND: Self = Self(0x00);
    pub const STATUS: Self = Self(0x01);
    pub const SIG_COUNT_VAL: Self = Self(0x02);
    pub const ACQ_CONFIG_REG: Self = Self(0x04);
    pub const VELOCITY: Self = Self(0x09);
    pub const PEAK_CORR: Self = Self(0x0c);
    pub const NOISE_PEAK: Self = Self(0x0d);
    pub const SIGNAL_STRENGTH: Self = Self(0x0e);
    pub const FULL_DELAY_HIGH: Self = Self(0x0f);
    pub const FULL_DELAY_LOW: Self = Self(0x10);
    pub const OUTER_LOOP_COUNT: Self = Self(0x11);
    pub const REF_COUNT_VAL: Self = Self(0x12);
    pub const LAST_DELAY_HIGH: Self = Self(0x14);
    pub const LAST_DELAY_LOW: Self = Self(0x15);
    pub const UNIT_ID_HIGH: Self = Self(0x16);
    pub const UNIT_ID_LOW: Self = Self(0x17);
    pub const I2C_ID_HIGH: Self = Self(0x18);
    pub const I2C_ID_LOW: Self = Self(0x19);
    pub const I2C_SEC_ADDR: Self = Self(0x1a);
    pub const THRESHOLD_BYPASS: Self = Self(0x1c);
    pub const I2C_CONIFG: Self = Self(0x1e);
    pub const COMMAND: Self = Self(0x40);
    pub const MEASURE_DELAY: Self = Self(0x45);
    pub const PEAK_BCK: Self = Self(0x4c);
    pub const CORR_DATA: Self = Self(0x52);
    pub const CORR_DATA_SIGN: Self = Self(0x53);
    pub const ACQ_SETTINGS: Self = Self(0x5d);
    pub const POWER_CONTROL: Self = Self(0x65);
}
pub struct AcqCommand(u8);
impl From<AcqCommand> for u8 {
    fn from(cr: AcqCommand) -> Self {
        cr.0
    }
}
impl AcqCommand {
    pub const RESET: Self = Self(0x00);
    pub const MEASURE_DIST_NO_CORRECTION: Self = Self(0x03);
    pub const MEASURE_DIST_CORRECTION: Self = Self(0x04);
}
pub struct Status(u8);
impl From<Status> for u8 {
    fn from(cr: Status) -> Self {
        cr.0
    }
}
impl From<Status> for [u8; 1] {
    fn from(cr: Status) -> Self {
        [cr.0]
    }
}
impl Status {
    pub const PROCESS_ERROR_FLAG: Self = Self(0b01000000);
    pub const HEALTH_FLAG: Self = Self(0b00100000);
    pub const SECONDARY_RETURN_FLAG: Self = Self(0b00010000);
    pub const INVALID_SIGANTURE_FLAG: Self = Self(0b00001000);
    pub const SIGNAL_OVERFLOW_FLAG: Self = Self(0b00000100);
    pub const REFERENCE_OVERFLOW_FLAG: Self = Self(0b00000010);
    pub const BUSY_FLAG: Self = Self(0b00000001);
}
pub struct AcqConfig(u8);
impl From<AcqConfig> for u8 {
    fn from(cr: AcqConfig) -> Self {
        cr.0
    }
}
impl AcqConfig {
    pub const REFERENCE_PROCESS: Self = Self(0b01000000);
    pub const BURST_FREE_DELAY: Self = Self(0b00100000);
    pub const REFERENCE_FILTER: Self = Self(0b00010000);
    pub const QUICK_TERMINATION: Self = Self(0b00001000);
    pub const AQUISITION_COUNT: Self = Self(0b00000100);
    pub const MODE_SELECT_PIN: Self = Self(0b00000011);
}
pub struct PowerCtl(u8);
impl From<PowerCtl> for u8 {
    fn from(cr: PowerCtl) -> Self {
        cr.0
    }
}
impl PowerCtl {
    pub const SLEEP: Self = Self(0b00000100);
    pub const RECEIVER_CIRCUIT: Self = Self(0b00000001);
}

pub struct GarminLidarV3 {
    address: u8,
}

impl GarminLidarV3 {
    /// Give address to configure a new i2c addresss
    pub fn new(address: Option<u8>, serial: &mut Usart0<MHz16>) -> GarminLidarV3 {
        let lidar_address = LIDAR_ADDRESS;
        if let Some(_) = address {
            let _ = ufmt::uwriteln!(
                serial,
                "Configure new address for garmin lidar not implemented"
            );
        }

        Self {
            address: lidar_address,
        }
    }
    pub fn start_auto_measurement(&mut self, i2c: &mut I2c, _serial: &mut Usart0<MHz16>) {
        // First we write to outer loop count
        let mut cmd: [u8; 2] = [ControlRegister::OUTER_LOOP_COUNT.into(), 0xff];
        let _ = i2c.write(self.address, &cmd);
        // Then we do the initial measurement
        cmd = [
            ControlRegister::ACQ_COMMAND.into(),
            AcqCommand::MEASURE_DIST_CORRECTION.into(),
        ];
        let _ = i2c.write(self.address, &cmd);
        // Now the auto command system is running on the lidar
    }
    pub fn read_last_measurement(&mut self, i2c: &mut I2c, _serial: &mut Usart0<MHz16>) -> u16 {
        let cmd: [u8; 1] = [Into::<u8>::into(ControlRegister::LAST_DELAY_HIGH) | 0b10000000];
        let mut data = [0u8; 2];
        let _ = ufmt::uwriteln!(_serial, "Reading i2c");
        let _ = i2c.write_read(self.address, &cmd, &mut data);

        u16::from_be_bytes(data)
    }
}

impl I2cLidarOps for GarminLidarV3 {
    fn read_distance_cm(&mut self, i2c: &mut I2c, serial: &mut Usart0<MHz16>) -> u16 {
        self.read_last_measurement(i2c, serial)
    }
}
