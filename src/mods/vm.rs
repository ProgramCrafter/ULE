// MovASM virtual machine in non-interactive mode

use std::cmp::min;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::thread;
use std::thread::JoinHandle;
// use std::sync::Mutex;

const MEMORY_SIZE: usize = 1048576;

// big array cannot be allocated on stack
// https://github.com/rust-lang/rust/issues/53827#issuecomment-576450631
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

struct Memory {
    buffer: Box<[u32; MEMORY_SIZE]>,
    io: (
        Cursor<Vec<u8>>,
        Cursor<Vec<u8>>
    )
}
impl Memory {
    fn load32(&self, address: usize) -> u32 {
        self.buffer[address]
    }

    fn load_opcode(&self, address: usize) -> (u16, u16) {
        let i32 = self.load32(address);
        (
            (i32 >> 16).try_into().unwrap(),
            (i32 & 0xFFFF).try_into().unwrap(),
        )
    }

    fn load64(&self, address: usize) -> u64 {
        self.buffer[address * 2] as u64 * 4294967296 + self.buffer[address * 2 + 1] as u64
    }

    fn store64(&mut self, address: usize, value: u64) {
        self.buffer[address * 2] = (value / 4294967296).try_into().unwrap();
        self.buffer[address * 2 + 1] = (value % 4294967296).try_into().unwrap();
    }

    fn store(&mut self, data: &[u8], base: usize) {
        if data.len() == 0 {return;}
        if data.len() % 4 != 0 {
            error!("mods VM: code len must be divided by 4 bytes");
        }
        
        let data_end = min(base + data.len() / 4, MEMORY_SIZE) - 1;
        for i in base..data_end {
            print!("Storing at position {}: ", i);
            self.buffer[i] = 0;
            for j in 0..4 {
                print!("{} ", (i - base) * 4 + j);
                self.buffer[i] *= 256;
                self.buffer[i] += data[(i - base) * 4 + j] as u32;
            }
            println!();
        }
    }
}

struct Registers {
    buffer: [i64; 36],
    triggers: HashMap<
        usize,
        (
            Vec<fn(usize, &mut [i64; 36], &mut Memory)>,
            Vec<fn(usize, &mut [i64; 36], &mut Memory)>,
        ),
    >,
}
impl Registers {
    fn get_triggers_pair(
        &mut self,
        index: usize,
    ) -> &mut (
        Vec<fn(usize, &mut [i64; 36], &mut Memory)>,
        Vec<fn(usize, &mut [i64; 36], &mut Memory)>,
    ) {
        self.triggers
            .entry(index)
            .or_insert((Vec::new(), Vec::new()))
    }

    fn init_triggers(&mut self) {
        fn add_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            buffer[2] = buffer[0] + buffer[1];
        }
        self.get_triggers_pair(0).1.push(add_trig);
        self.get_triggers_pair(1).1.push(add_trig);
        self.get_triggers_pair(2).0.push(add_trig);

        fn sub_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            buffer[5] = buffer[3] - buffer[4];
        }
        self.get_triggers_pair(3).1.push(sub_trig);
        self.get_triggers_pair(4).1.push(sub_trig);
        self.get_triggers_pair(5).0.push(sub_trig);

        fn mul_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            buffer[8] = buffer[6] * buffer[7];
        }
        self.get_triggers_pair(6).1.push(mul_trig);
        self.get_triggers_pair(7).1.push(mul_trig);
        self.get_triggers_pair(8).0.push(mul_trig);

        fn div_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            let div0 = buffer[9];
            let div1 = buffer[10];
            buffer[11] = if div1 != 0 { div0 / div1 } else { div0 };
            buffer[12] = if div1 != 0 { div0 % div1 } else { 0 };
        }
        self.get_triggers_pair(9).1.push(div_trig);
        self.get_triggers_pair(10).1.push(div_trig);
        self.get_triggers_pair(11).0.push(div_trig);
        self.get_triggers_pair(12).0.push(div_trig);

        fn tlt_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            buffer[15] = if buffer[13] < buffer[14] { 1 } else { 0 };
        }
        self.get_triggers_pair(13).1.push(tlt_trig);
        self.get_triggers_pair(14).1.push(tlt_trig);
        self.get_triggers_pair(15).0.push(tlt_trig);

        // 16 - cio - removed in non-interactive mode
        // 17 - io0 - selection of device
        fn io_trig(trig: usize, buffer: &mut [i64; 36], memory: &mut Memory) {
            if trig == 1 {
                memory.io.1.write(&[buffer[18].try_into().unwrap()]);
            } else {
                let mut buf: [u8; 1] = [10; 1];
                
                // #[allow(unused_result)]
                memory.io.0.read(&mut buf);
                
                buffer[19] = buf[0].into();
            }
        }
        self.get_triggers_pair(18).1.push(io_trig);
        self.get_triggers_pair(19).0.push(io_trig);

        fn atz_trig(_trig: usize, buffer: &mut [i64; 36], _memory: &mut Memory) {
            buffer[23] = if buffer[20] == 0 {
                buffer[21]
            } else {
                buffer[22]
            };
        }
        self.get_triggers_pair(20).1.push(atz_trig);
        self.get_triggers_pair(21).1.push(atz_trig);
        self.get_triggers_pair(22).1.push(atz_trig);
        self.get_triggers_pair(23).0.push(atz_trig);

        fn mem_trig(trig: usize, buffer: &mut [i64; 36], memory: &mut Memory) {
            if trig == 1 {
                memory.store64(
                    buffer[26].try_into().unwrap(),
                    buffer[24].try_into().unwrap(),
                );
            } else {
                buffer[24] = memory
                    .load64(buffer[26].try_into().unwrap())
                    .try_into()
                    .unwrap();
            }
        }
        self.get_triggers_pair(24).0.push(mem_trig);
        self.get_triggers_pair(24).1.push(mem_trig);
        self.get_triggers_pair(26).1.push(mem_trig);
    }

    fn set(&mut self, index: usize, value: i64, memory: &mut Memory) {
        self.buffer[index] = value;

        match &self.triggers.get(&index) {
            Some(trigs) => {
                let buf = &mut (self.buffer);
                for callback in trigs.1.iter() {
                    callback(1, buf, memory);
                }
            }
            None => {}
        }
    }

    fn get(&mut self, index: usize, memory: &mut Memory) -> i64 {
        match &self.triggers.get(&index) {
            Some(trigs) => {
                let buf = &mut (self.buffer);
                for callback in trigs.0.iter() {
                    callback(0, buf, memory);
                }
            }
            None => {}
        }
        self.buffer[index]
    }
}

pub struct Machine {
    regs: Registers,
    mem: Memory
}
impl Machine {
    pub fn new(code: &str) -> Machine {
        let mut machine = Machine {
            regs: Registers {
                buffer: [0; 36],
                triggers: HashMap::new(),
            },
            mem: Memory {
                buffer: box_array![0; MEMORY_SIZE],
                io: (Cursor::new(Vec::new()), Cursor::new(Vec::new())),
            }
        };
        
        machine.mem.store(code.as_bytes(), 0);
        machine.regs.init_triggers();
        machine
    }
    
    fn execute(&mut self) {
        loop {
            let addr = self.regs.buffer[27] as usize;

            if addr >= MEMORY_SIZE {
                break;
            }

            let (src, dst) = self.mem.load_opcode(addr);

            let val = if src & 0x8000 != 0 {
                (src & 0x7FFF) as i64
            } else {
                self.regs.get(src.try_into().unwrap(), &mut self.mem)
            };
            self.regs.buffer[27] = (addr + 1).try_into().unwrap();
            self.regs.set(dst.try_into().unwrap(), val, &mut self.mem);
        }
    }
    
    pub fn execute_threaded(self) -> JoinHandle<Machine> {
        // self is moved from calling code
        thread::spawn(|| {
            let mut machine = self; // machine is moved again
            
            machine.execute();
            
            machine // returning, in case someone wants to read the output
        })
    }
    
    #[allow(dead_code)]
    pub fn write_into_vm(&mut self, s: &str) {
        self.mem.io.0.write_all(s.as_bytes()).unwrap();
    }
    
    #[allow(dead_code)]
    pub fn read_from_vm(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        self.mem.io.1.read_to_end(&mut buf).unwrap();
        buf
    }
    
    pub fn read_str_from_vm(&mut self) -> String {
        let mut buf: String = String::new();
        self.mem.io.1.read_to_string(&mut buf).unwrap();
        buf
    }
}
