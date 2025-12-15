use crate::sensirion::Cmd;

pub const CMD_GET_SERIAL_NUMBER: Cmd = [0x36, 0x82];
pub const CMD_TURN_HEATER_OFF: Cmd = [0x36, 0x15];
pub const CMD_EXECUTE_SELF_TEST: Cmd = [0x28, 0x0e];
pub const CMD_MEASURE_RAW_SIGNAL: Cmd = [0x26, 0x02];
