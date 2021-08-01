pub struct Timer {
    clock: u16,
    timer_counter: i16,
    timer: u8,
    modulo: u8,
    control: u8,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            clock: 0,
            timer_counter: 1024,
            timer: 0,
            modulo: 0,
            control: 0,
        }
    }
}

impl Timer {
    pub fn step(&mut self, cycles: usize) -> bool {
        // TODO: Don't update when in STOP mode.
        self.clock = self.clock.wrapping_add(cycles as u16);
        // TODO: According to this schematic
        //       https://gbdev.io/pandocs/#timer-global-circuit
        //       timer_counter is coupled to clock.
        if !self.is_timer_enabled() {
            return false;
        }
        let mut interrupt = false;
        self.timer_counter -= cycles as i16;
        // TODO: Can this overflow multiple times when processing a slow
        //       instruction?
        while self.timer_counter <= 0 {
            self.timer_counter += self.ticks_per_timer_increment();
            let (new_timer, overflow) = self.timer.overflowing_add(1);
            self.timer = if !overflow {
                new_timer
            } else {
                interrupt = true;
                self.modulo
            };
        }
        interrupt
    }

    pub fn get_divider(&self) -> u8 {
        (self.clock >> 8) as u8
    }

    pub fn reset_divider(&mut self) {
        self.clock = 0;
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
        // TODO: What happens to the unused bits 3–7?
        self.control = value;
    }

    fn is_timer_enabled(&self) -> bool {
        self.control & 0x4 != 0
    }

    fn ticks_per_timer_increment(&self) -> i16 {
        // https://gbdev.io/pandocs/#ff07-tac-timer-control-r-w
        match self.control & 0x3 {
            0 => 1024,
            1 =>   16,
            2 =>   64,
            3 =>  256,
            _ => unreachable!(),
        }
    }
}
