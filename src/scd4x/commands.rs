use crate::sensirion::Cmd;

// Basic commands
pub const START_PERIODIC_MEASUREMENTS: Cmd = [0x21, 0xb1];
pub const READ_MEASUREMENT: Cmd = [0xec, 0x05];
pub const STOP_PERIODIC_MEASUREMENTS: Cmd = [0x3f, 0x86];

// On-chip output signal compensation
pub const SET_TEMPERATURE_OFFSET: Cmd = [0x24, 0x1d];
pub const GET_TEMPERATURE_OFFSET: Cmd = [0x23, 0x18];
pub const SET_SENSOR_ALTITUDE: Cmd = [0x24, 0x27];
pub const GET_SENSOR_ALTITUDE: Cmd = [0x23, 0x22];
pub const SET_AMBIENT_PRESSURE: Cmd = [0xe0, 0x00];
pub const GET_AMBIENT_PRESSURE: Cmd = [0xe0, 0x00];

// Field calibration
pub const PERFORM_FORCED_RECALIBRATION: Cmd = [0x36, 0x2f];
pub const SET_AUTOMATIC_SELF_CALIBRATION_ENABLED: Cmd = [0x24, 0x16];
pub const GET_AUTOMATIC_SELF_CALIBRATION_ENABLED: Cmd = [0x23, 0x13];
pub const SET_AUTOMATIC_SELF_CALIBRATION_TARGET: Cmd = [0x24, 0x3a];
pub const GET_AUTOMATIC_SELF_CALIBRATION_TARGET: Cmd = [0x23, 0x3f];

// Low power periodic measurement mode
pub const START_LOW_POWER_PERIODIC_MEASUREMENT: Cmd = [0x21, 0xac];
pub const GET_DATA_READY_STATUS: Cmd = [0xe4, 0xb8];

// Advanced features
pub const PERSIST_SETTINGS: Cmd = [0x36, 0x15];
pub const GET_SERIAL_NUMBER: Cmd = [0x36, 0x82];
pub const PERFORM_SELF_TEST: Cmd = [0x36, 0x39];
pub const PERFORM_FACTORY_RESET: Cmd = [0x36, 0x32];
pub const REINIT: Cmd = [0x36, 0x46];
pub const GET_SENSOR_VARIANT: Cmd = [0x20, 0x2f];

// Single shot measurement mode (SCD41 and SCD43)
pub const MEASURE_SINGLE_SHOT: Cmd = [0x21, 0x9d];
pub const MEASURE_SINGLE_SHOT_RHT_ONLY: Cmd = [0x21, 0x96];
pub const POWER_DOWN: Cmd = [0x36, 0xe0];
pub const WAKE_UP: Cmd = [0x36, 0xf6];
pub const SET_AUTOMATIC_SELF_CALIBRATION_INITIAL_PERIOD: Cmd = [0x24, 0x45];
pub const GET_AUTOMATIC_SELF_CALIBRATION_INITIAL_PERIOD: Cmd = [0x23, 0x40];
pub const SET_AUTOMATIC_SELF_CALIBRATION_STANDARD_PERIOD: Cmd = [0x24, 0x4e];
pub const GET_AUTOMATIC_SELF_CALIBRATION_STANDARD_PERIOD: Cmd = [0x23, 0x4b];
