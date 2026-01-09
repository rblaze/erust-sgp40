use crate::sensirion::Cmd;

pub const GET_SERIAL_NUMBER: Cmd = [0x36, 0x82];
pub const TURN_HEATER_OFF: Cmd = [0x36, 0x15];
pub const EXECUTE_SELF_TEST: Cmd = [0x28, 0x0e];
pub const MEASURE_RAW_SIGNAL: Cmd = [0x26, 0x0F];
