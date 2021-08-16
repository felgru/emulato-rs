pub struct Timer {
    clock: u16,
    timer_counter: u16,
    timer_trigger: u16,
    timer_trigger_bit: bool,
    timer: u8,
    modulo: u8,
    control: u8,
    stopped: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            clock: 0,
            timer_counter: 0,
            timer_trigger: 1 << 9,
            timer_trigger_bit: false,
            timer: 0,
            modulo: 0,
            control: 0xF8,  // Only lowest 3 bits are used, rest is 1.
            stopped: false,
        }
    }
}

impl Timer {
    pub fn step(&mut self, cycles: usize) -> bool {
        if self.stopped {
            return false;
        }
        self.clock = self.clock.wrapping_add(cycles as u16);
        // TODO: Can the timer increment multiple times when processing a slow
        //       instruction?
        // Coupling of clock and timer according to
        // https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html
        let new_timer_trigger_bit
            = self.is_timer_enabled()
              && (self.clock & self.timer_trigger) != 0;
        let interrupt = if self.timer_trigger_bit && !new_timer_trigger_bit {
            let (new_timer, overflow) = self.timer.overflowing_add(1);
            let mut interrupt = false;
            self.timer = if !overflow {
                new_timer
            } else {
                interrupt = true;
                self.modulo
            };
            interrupt
        } else {
            false
        };
        self.timer_trigger_bit = new_timer_trigger_bit;
        interrupt
    }

    pub fn stop_clock(&mut self) {
        self.stopped = true;
        self.reset_divider();
    }

    pub fn start_clock(&mut self) {
        self.stopped = false;
    }

    pub fn get_divider(&self) -> u8 {
        (self.clock >> 8) as u8
    }

    pub fn reset_divider(&mut self) {
        self.clock = 0;
        self.timer_counter = 0;
    }

    pub fn get_timer(&self) -> u8 {
        self.timer
    }

    pub fn set_timer(&mut self, value: u8) {
        self.timer = value;
    }

    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, value: u8) {
        self.modulo = value;
    }

    pub fn get_control(&self) -> u8 {
        self.control
    }

    pub fn set_control(&mut self, value: u8) {
        // The unused bits 3–7 are always 1.
        self.control = value | 0xF8;
        // TODO: There are probably weird corner cases to handle
        //       if the new trigger makes the timer overflow.
        self.timer_trigger = self.get_timer_trigger_bit();
    }

    fn is_timer_enabled(&self) -> bool {
        self.control & 0x4 != 0
    }

    fn get_timer_trigger_bit(&self) -> u16 {
        // https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff07---tac---timer-control-rw
        match self.control & 0x3 {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            3 => 1 << 7,
            _ => unreachable!(),
        }
    }
}
