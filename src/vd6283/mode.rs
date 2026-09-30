use super::{
    device::Vd6283,
    error::{Error, Result},
    regs,
    types::{AlsData, Channel, DeviceState, FlickerOutput, Mode, CHANNEL_COUNT},
};

impl<I2C> Vd6283<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn start_single_shot(&mut self, channels: u8) -> Result<()> {
        self.start_als(channels, Mode::AlsSingleShot)
    }

    pub fn start_synchronous(&mut self, channels: u8) -> Result<()> {
        self.start_als(channels, Mode::AlsSynchronous)
    }

    pub fn stop_als(&mut self) -> Result<()> {
        if !matches!(
            self.state,
            DeviceState::AlsRunning | DeviceState::BothRunning
        ) {
            return Err(Error::NotStarted);
        }
        self.write_reg(regs::REG_ALS_CONTROL, 0)?;
        self.disable_channels(Mode::AlsSingleShot)?;
        self.state = if self.flicker_channels == 0 {
            DeviceState::Init
        } else {
            DeviceState::FlickerRunning
        };
        self.als_mode = None;
        Ok(())
    }

    pub fn start_flicker(&mut self, channel: Channel, output: FlickerOutput) -> Result<()> {
        if !matches!(self.state, DeviceState::Init | DeviceState::AlsRunning) {
            return Err(if self.state == DeviceState::FlickerRunning {
                Error::AlreadyStarted
            } else {
                Error::InvalidDeviceId
            });
        }
        if self.flicker_channels != 0 || self.als_channels & channel.mask() != 0 {
            return Err(Error::InvalidParams);
        }

        self.enable_channels(Mode::Flicker, channel.mask())?;
        self.configure_flicker_output(output)?;
        let index = channel.index();
        let ac_channel = if index == 5 { 1 } else { index + 2 };
        let value = ((ac_channel as u8) << 1) | regs::AC_EXTRACTOR_ENABLE;
        if let Err(error) =
            self.update_ac_mode(regs::AC_CHANNEL_SELECT_MASK | regs::AC_EXTRACTOR, value)
        {
            let _ = self.disable_channels(Mode::Flicker);
            return Err(error);
        }

        self.flicker_output = output;
        self.state = if self.als_channels == 0 {
            DeviceState::FlickerRunning
        } else {
            DeviceState::BothRunning
        };
        Ok(())
    }

    pub fn stop_flicker(&mut self) -> Result<()> {
        if !matches!(
            self.state,
            DeviceState::FlickerRunning | DeviceState::BothRunning
        ) {
            return Err(Error::NotStarted);
        }
        self.update_ac_mode(regs::AC_EXTRACTOR, regs::AC_EXTRACTOR_DISABLE)?;
        self.disable_flicker_output()?;
        self.disable_channels(Mode::Flicker)?;
        self.state = if self.als_channels == 0 {
            DeviceState::Init
        } else {
            DeviceState::AlsRunning
        };
        Ok(())
    }

    pub fn is_data_ready(&mut self) -> Result<bool> {
        Ok(self.read_reg(regs::REG_IRQ_CTRL_STATUS)? & 0x02 == 0)
    }

    pub fn read_als(&mut self, requested_channels: u8) -> Result<Option<AlsData>> {
        if !matches!(
            self.state,
            DeviceState::AlsRunning | DeviceState::BothRunning
        ) {
            return Err(Error::NotStarted);
        }
        validate_channel_mask(requested_channels, false)?;
        if !self.is_data_ready()? {
            return Ok(None);
        }

        let channels = requested_channels & self.als_channels;
        let mut raw = [0u8; CHANNEL_COUNT * 4];
        self.read_regs(regs::channel_data_reg(0), &mut raw)?;

        let mut als = AlsData {
            channels,
            ..AlsData::default()
        };
        for channel in 0..CHANNEL_COUNT {
            if channels & (1 << channel) == 0 {
                continue;
            }
            let offset = channel * 4;
            als.count_value_raw[channel] = (u32::from(raw[offset]) << 24)
                | (u32::from(raw[offset + 1]) << 16)
                | (u32::from(raw[offset + 2]) << 8)
                | u32::from(raw[offset + 3]);
        }
        self.apply_calibration(&mut als);
        self.acknowledge_irq()?;
        Ok(Some(als))
    }

    fn start_als(&mut self, channels: u8, mode: Mode) -> Result<()> {
        if matches!(
            self.state,
            DeviceState::AlsRunning | DeviceState::BothRunning
        ) {
            return Err(Error::AlreadyStarted);
        }
        validate_channel_mask(channels, true)?;
        self.enable_channels(mode, channels)?;

        let command = match mode {
            Mode::AlsSingleShot => regs::ALS_START,
            Mode::AlsSynchronous => {
                regs::ALS_START | regs::ALS_CONTINUOUS | regs::ALS_CONTINUOUS_SLAVED
            }
            Mode::Flicker => return Err(Error::InvalidParams),
        };

        let was_flicker = self.state == DeviceState::FlickerRunning;
        if was_flicker {
            self.update_ac_mode(regs::AC_EXTRACTOR, regs::AC_EXTRACTOR_DISABLE)?;
        }
        if let Err(error) = self.write_reg(regs::REG_ALS_CONTROL, command) {
            let _ = self.disable_channels(mode);
            return Err(error);
        }
        if was_flicker {
            self.update_ac_mode(regs::AC_EXTRACTOR, regs::AC_EXTRACTOR_ENABLE)?;
        }

        self.als_mode = Some(mode);
        self.state = if self.flicker_channels == 0 {
            DeviceState::AlsRunning
        } else {
            DeviceState::BothRunning
        };
        Ok(())
    }

    fn enable_channels(&mut self, mode: Mode, channels: u8) -> Result<()> {
        let active = self.als_channels | self.flicker_channels;
        if channels & active != 0 {
            return Err(Error::InvalidParams);
        }
        let new_dc = self.dc_channels_enabled | (channels & regs::DC_CHANNELS_MASK);
        if new_dc != self.dc_channels_enabled {
            self.write_reg(regs::REG_DC_ENABLE, new_dc)?;
        }
        if channels & regs::AC_CHANNELS_MASK != 0 && !self.ac_enabled {
            if let Err(error) = self.write_reg(regs::REG_AC_ENABLE, 1) {
                let _ = self.write_reg(regs::REG_DC_ENABLE, self.dc_channels_enabled);
                return Err(error);
            }
            self.ac_enabled = true;
        }
        self.dc_channels_enabled = new_dc;
        if mode == Mode::Flicker {
            self.flicker_channels = channels;
        } else {
            self.als_channels = channels;
        }
        Ok(())
    }

    fn disable_channels(&mut self, mode: Mode) -> Result<()> {
        let channels = if mode == Mode::Flicker {
            self.flicker_channels
        } else {
            self.als_channels
        };
        let new_dc = self.dc_channels_enabled & !(channels & regs::DC_CHANNELS_MASK);
        if new_dc != self.dc_channels_enabled {
            self.write_reg(regs::REG_DC_ENABLE, new_dc)?;
        }
        if channels & regs::AC_CHANNELS_MASK != 0 {
            self.write_reg(regs::REG_AC_ENABLE, 0)?;
            self.ac_enabled = false;
        }
        self.dc_channels_enabled = new_dc;
        if mode == Mode::Flicker {
            self.flicker_channels = 0;
        } else {
            self.als_channels = 0;
        }
        Ok(())
    }

    fn configure_flicker_output(&mut self, output: FlickerOutput) -> Result<()> {
        let (pdm_output, pdm_clock, interrupt, selection, spare) = match output {
            FlickerOutput::DigitalPdm => (
                regs::PDM_OUTPUT_GPIO1,
                regs::PDM_CLOCK_EXTERNAL,
                0x01,
                0x0f,
                regs::SPARE_0_INPUT_GPIO2,
            ),
            FlickerOutput::Analog => (
                regs::PDM_OUTPUT_GPIO2,
                regs::PDM_CLOCK_INTERNAL,
                0x00,
                0x00,
                regs::SPARE_0_OUTPUT_GPIO2,
            ),
        };
        self.update_ac_mode(
            regs::PDM_SELECT_OUTPUT | regs::PDM_SELECT_CLOCK,
            pdm_output | pdm_clock,
        )?;
        self.write_reg(regs::REG_INTERRUPT_CONFIG, interrupt)?;
        self.write_reg(regs::REG_DTEST_SELECTION, selection)?;
        self.write_reg(regs::REG_SPARE_0, spare)
    }

    fn disable_flicker_output(&mut self) -> Result<()> {
        self.write_reg(regs::REG_SPARE_0, regs::SPARE_0_INPUT_GPIO2)?;
        self.update_ac_mode(
            regs::PDM_SELECT_OUTPUT | regs::PDM_SELECT_CLOCK,
            regs::PDM_OUTPUT_GPIO1 | regs::PDM_CLOCK_INTERNAL,
        )?;
        self.write_reg(regs::REG_INTERRUPT_CONFIG, 0)?;
        self.write_reg(regs::REG_DTEST_SELECTION, 0)
    }

    fn acknowledge_irq(&mut self) -> Result<()> {
        self.write_reg(regs::REG_IRQ_CTRL_STATUS, 1)?;
        self.write_reg(regs::REG_IRQ_CTRL_STATUS, 0)
    }
}

fn validate_channel_mask(mask: u8, require_one: bool) -> Result<()> {
    if mask & !((1 << CHANNEL_COUNT) - 1) != 0 || (require_one && mask == 0) {
        return Err(Error::InvalidParams);
    }
    Ok(())
}
