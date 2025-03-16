use core::fmt;
use x86_64::instructions::port::Port;

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    base_port: u16 // 存储串口基地址，通过对其的偏移量访问其他寄存器
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            base_port: port
        }
    }

    /// Initializes the serial port.
    pub fn init(&self) {
        // FIXME: Initialize the serial port

        // Interrupt Enable Reg
        let mut ier = Port::<u8>::new(self.base_port + 1);
        
        // Line Control Reg. The most significant bit of this register is the DLAB.
        let mut lcr = Port::<u8>::new(self.base_port + 3);
        
        // With DLAB set to 1, this is the least significant byte of the divisor value for setting the baud rate.
        let mut div_lo = Port::<u8>::new(self.base_port + 0);
        
        // With DLAB set to 1, this is the most significant byte of the divisor value.
        let mut div_hi = Port::<u8>::new(self.base_port + 1);

        // FIFO control registers
        let mut fcr = Port::<u8>::new(self.base_port + 2);

        // Modem Control Reg
        let mut mcr = Port::<u8>::new(self.base_port + 4);

        unsafe {
            ier.write(0x00); // 首先关掉所有中断
            lcr.write(0x80); // lcr 的最高为为DLAB，设置为1，从而设置波特率
            div_lo.write(0x03);
            div_hi.write(0x00); // 设置分频因子为3
            lcr.write(0x03); // 设置数据格式，8-bit 数据位，无校验位，1 位停止位
            fcr.write(0xC7); // 启用 FIF
            mcr.write(0x0B); // 开启 IRQ 且设置 DTR/RTS

            // loopback mode, 测试 UART 是否正常工作
            Port::<u8>::new(self.base_port + 4).write(0x1E);
            
            // test
            Port::<u8>::new(self.base_port + 0).write(0xAE);
            if Port::<u8>::new(self.base_port + 0).read() != 0xAE {
                panic!{"serial is faulty"};
            }

            // normal mode
            Port::<u8>::new(self.base_port + 4).write(0x0F);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        let mut lsr = Port::<u8>::new(self.base_port + 5);
        let mut tb = Port::<u8>::new(self.base_port + 0);
        unsafe {
            while lsr.read() & 0x20 == 0 {};
            tb.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        let mut lsr = Port::<u8>::new(self.base_port + 5);
        let mut rb = Port::<u8>::new(self.base_port + 0);
        unsafe {
            // while lsr.read() & 1 == 0 {};
            // no wait
            if lsr.read() & 1 != 0 {
                Some(rb.read())
            } else {
                None
            }
            
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
