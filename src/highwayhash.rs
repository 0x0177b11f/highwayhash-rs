fn zipper_merge_and_add(v1: u64, v0: u64, add1: u64, add0: u64) -> (u64, u64) {
    let new_add1 = add1.wrapping_add(
        (v1 & 0xff000000u64 | v0 & 0xff00000000u64) >> 24
            | v1 & 0xff0000u64
            | (v1 & 0xff0000000000u64) >> 16
            | (v1 & 0xff00u64) << 24
            | (v0 & 0xff000000000000u64) >> 8
            | (v1 & 0xffu64) << 48
            | v0 & 0xff00000000000000u64,
    );
    let new_add0 = add0.wrapping_add(
        (v0 & 0xff000000u64 | v1 as u64 & 0xff00000000u64) >> 24
            | (v0 as u64 & 0xff0000000000u64 | v1 as u64 & 0xff000000000000u64) >> 16
            | v0 as u64 & 0xff0000u64
            | (v0 as u64 & 0xff00u64) << 32
            | (v1 as u64 & 0xff00000000000000u64) >> 8
            | (v0 << 56),
    );

    (new_add1, new_add0)
}

fn read64(src: &[u8]) -> u64 {
    return src[0] as u64
        | (src[1] as u64) << 8
        | (src[2] as u64) << 16
        | (src[3] as u64) << 24
        | (src[4] as u64) << 32
        | (src[5] as u64) << 40
        | (src[6] as u64) << 48
        | (src[7] as u64) << 56;
}

fn rotate32_by(count: usize, lanes: &[u64; 4]) -> [u64; 4] {
    let mut new_lanes: [u64; 4] = [0; 4];
    let mut i: usize = 0;
    while i < 4 {
        let half0: u32 = (lanes[i] & 0xffffffffu32 as u64) as u32;
        let half1: u32 = (lanes[i] >> 32) as u32;

        new_lanes[i] = (half0 << count | half0 >> (32u32).wrapping_sub(count as u32)) as u64;
        new_lanes[i] = new_lanes[i]
            | ((half1 << count | half1 >> (32u32).wrapping_sub(count as u32)) as u64) << 32u32;
        i += 1
    }

    new_lanes
}

fn permute(v: [u64; 4]) -> [u64; 4] {
    let mut permuted: [u64; 4] = [0; 4];
    permuted[0] = (v[2] >> 32) | (v[2] << 32);
    permuted[1] = (v[3] >> 32) | (v[3] << 32);
    permuted[2] = (v[0] >> 32) | (v[0] << 32);
    permuted[3] = (v[1] >> 32) | (v[1] << 32);
    permuted
}

fn modular_reduction(a3_unmasked: u64, a2: u64, a1: u64, a0: u64) -> (u64, u64) {
    let a3 = a3_unmasked & 0x3FFFFFFFFFFFFFFFu64;
    let new_m1 = a1 ^ ((a3 << 1) | (a2 >> 63)) ^ ((a3 << 2) | (a2 >> 62));
    let new_m0 = a0 ^ (a2 << 1) ^ (a2 << 2);

    (new_m1, new_m0)
}

#[derive(Debug, Copy, Clone)]
pub struct HighwayHashState {
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl HighwayHashState {
    fn new() -> Self {
        let state: HighwayHashState = HighwayHashState {
            v0: [0; 4],
            v1: [0; 4],
            mul0: [0; 4],
            mul1: [0; 4],
        };
        state
    }

    fn reset(&mut self, key: [u64; 4]) {
        self.mul0[0] = 0xdbe6d5d5fe4cce2fu64;
        self.mul0[1] = 0xa4093822299f31d0u64;
        self.mul0[2] = 0x13198a2e03707344u64;
        self.mul0[3] = 0x243f6a8885a308d3u64;
        self.mul1[0] = 0x3bd39e10cb0ef593u64;
        self.mul1[1] = 0xc0acf169b5f18a8cu64;
        self.mul1[2] = 0xbe5466cf34e90c6cu64;
        self.mul1[3] = 0x452821e638d01377u64;

        self.v0[0] = self.mul0[0] ^ key[0];
        self.v0[1] = self.mul0[1] ^ key[1];
        self.v0[2] = self.mul0[2] ^ key[2];
        self.v0[3] = self.mul0[3] ^ key[3];
        self.v1[0] = self.mul1[0] ^ ((key[0] >> 32) | (key[0] << 32));
        self.v1[1] = self.mul1[1] ^ ((key[1] >> 32) | (key[1] << 32));
        self.v1[2] = self.mul1[2] ^ ((key[2] >> 32) | (key[2] << 32));
        self.v1[3] = self.mul1[3] ^ ((key[3] >> 32) | (key[3] << 32));
    }

    fn update(&mut self, lanes: &mut [u64; 4]) {
        for i in 0..4 {
            self.v1[i] = self.v1[i].wrapping_add(self.mul0[i].wrapping_add(lanes[i]));
            self.mul0[i] ^= (self.v1[i] & 0xffffffffu32 as u64).wrapping_mul(self.v0[i] >> 32);
            self.v0[i] = self.v0[i].wrapping_add(self.mul1[i]);
            self.mul1[i] ^= (self.v0[i] & 0xffffffffu32 as u64).wrapping_mul(self.v1[i] >> 32);
        }

        let new_v0_01 = zipper_merge_and_add(self.v1[1], self.v1[0], self.v0[1], self.v0[0]);
        self.v0[1] = new_v0_01.0;
        self.v0[0] = new_v0_01.1;

        let new_v0_32 = zipper_merge_and_add(self.v1[3], self.v1[2], self.v0[3], self.v0[2]);
        self.v0[3] = new_v0_32.0;
        self.v0[2] = new_v0_32.1;

        let new_v1_10 = zipper_merge_and_add(self.v0[1], self.v0[0], self.v1[1], self.v1[0]);
        self.v1[1] = new_v1_10.0;
        self.v1[0] = new_v1_10.1;

        let new_v1_32 = zipper_merge_and_add(self.v0[3], self.v0[2], self.v1[3], self.v1[2]);
        self.v1[3] = new_v1_32.0;
        self.v1[2] = new_v1_32.1;
    }

    fn update_packet(&mut self, packet: &[u8]) {
        let mut lanes: [u64; 4] = [0; 4];
        lanes[0] = read64(packet);
        lanes[1] = read64(&packet[8..]);
        lanes[2] = read64(&packet[16..]);
        lanes[3] = read64(&packet[24..]);
        self.update(&mut lanes);
    }

    fn update_remainder(&mut self, bytes: &[u8], size_mod32: usize) {
        let size_mod4: usize = size_mod32 & 3;
        let remainder = size_mod32 & !3;

        let mut packet: [u8; 32] = [0; 32];
        for i in 0..4 {
            self.v0[i] = self.v0[i].wrapping_add(((size_mod32 << 32) + size_mod32) as u64);
        }
        self.v1 = rotate32_by(size_mod32, &mut self.v1);

        for i in 0..remainder {
            packet[i] = bytes[i];
        }

        if (size_mod32 & 16) != 0 {
            for i in 0..4 {
                packet[28 + i] = bytes[remainder
                    .wrapping_add(i)
                    .wrapping_add(size_mod4)
                    .wrapping_sub(4)];
            }
        } else {
            if size_mod4 != 0 {
                packet[16 + 0] = bytes[remainder + 0];
                packet[16 + 1] = bytes[remainder.wrapping_add(size_mod4 >> 1)];
                packet[16 + 2] = bytes[remainder.wrapping_add(size_mod4.wrapping_sub(1))];
            }
        }
        self.update_packet(&packet);
    }

    fn permute_and_update(&mut self) {
        let mut permuted: [u64; 4] = permute(self.v0);
        self.update(&mut permuted);
    }

    fn finalize64(&mut self) -> u64 {
        for _ in 0..4 {
            self.permute_and_update();
        }
        self.v0[0]
            .wrapping_add(self.v1[0])
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.mul1[0])
    }

    fn finalize128(&mut self) -> [u64; 2] {
        for _ in 0..6 {
            self.permute_and_update();
        }
        let mut hash: [u64; 2] = [0; 2];
        hash[0] = self.v0[0]
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.v1[2])
            .wrapping_add(self.mul1[2]);
        hash[1] = self.v0[1]
            .wrapping_add(self.mul0[1])
            .wrapping_add(self.v1[3])
            .wrapping_add(self.mul1[3]);

        hash
    }

    fn finalize256(&mut self) -> [u64; 4] {
        for _ in 0..10 {
            self.permute_and_update();
        }
        let mut hash: [u64; 4] = [0; 4];
        let new_hash10 = modular_reduction(
            self.v1[1].wrapping_add(self.mul1[1]),
            self.v1[0].wrapping_add(self.mul1[0]),
            self.v0[1].wrapping_add(self.mul0[1]),
            self.v0[0].wrapping_add(self.mul0[0]),
        );
        hash[1] = new_hash10.0;
        hash[0] = new_hash10.1;

        let new_hash32 = modular_reduction(
            self.v1[3].wrapping_add(self.mul1[3]),
            self.v1[2].wrapping_add(self.mul1[2]),
            self.v0[3].wrapping_add(self.mul0[3]),
            self.v0[2].wrapping_add(self.mul0[2]),
        );

        hash[3] = new_hash32.0;
        hash[2] = new_hash32.1;

        hash
    }

    fn process_all(&mut self, data: &Vec<u8>, key: [u64; 4]) {
        self.reset(key);

        let data_size = data.len();
        let mut offset: usize = 0;
        while (offset + 32) <= data_size {
            self.update_packet(&data[offset..]);
            offset += 32;
        }

        if (data_size & 31) != 0 {
            self.update_remainder(&data[offset..], data_size & 31);
        }
    }
}

pub fn highway_hash64(data: &Vec<u8>, key: [u64; 4]) -> u64 {
    let mut state = HighwayHashState::new();
    state.process_all(data, key);
    state.finalize64()
}

pub fn highway_hash128(data: &Vec<u8>, key: [u64; 4]) -> [u64; 2] {
    let mut state = HighwayHashState::new();
    state.process_all(data, key);
    state.finalize128()
}

pub fn highway_hash256(data: &Vec<u8>, key: [u64; 4]) -> [u64; 4] {
    let mut state = HighwayHashState::new();
    state.process_all(data, key);
    state.finalize256()
}

#[derive(Copy, Clone)]
pub struct HighwayHashCat {
    state: HighwayHashState,
    packet: [u8; 32],
    num: usize,
}

impl HighwayHashCat {
    pub fn new(key: [u64; 4]) -> Self {
        let mut hash_cat = HighwayHashCat {
            state: HighwayHashState::new(),
            packet: [0; 32],
            num: 0,
        };

        hash_cat.state.reset(key);
        hash_cat
    }

    pub fn append(&mut self, data: &[u8]) {
        let mut data_size: usize = data.len();
        let mut offset: usize = 0;

        if self.num != 0 {
            let num_add: usize = if data_size > (32 as usize).wrapping_sub(self.num) {
                (32 as usize).wrapping_sub(self.num)
            } else {
                data_size
            };

            let mut i: usize = 0;
            while i < num_add {
                self.packet[self.num.wrapping_add(i)] = data[i];
                i = i.wrapping_add(1)
            }
            self.num = self.num.wrapping_add(num_add);

            data_size = (data_size).wrapping_sub(num_add);
            offset = num_add;

            if self.num == 32 {
                self.state.update_packet(&self.packet[..]);
                self.num = 0
            }
        }

        while data_size >= 32 {
            self.state.update_packet(&data[offset..]);

            data_size = (data_size).wrapping_sub(32);
            offset = 32;
        }

        let mut i: usize = 0;
        while i < data_size {
            self.packet[self.num] = data[offset.wrapping_add(i)];
            self.num += 1;
            i = i.wrapping_add(1)
        }
    }

    pub fn finish64(&self) -> u64 {
        let mut copy: HighwayHashState = self.state;
        if self.num != 0 {
            copy.update_remainder(&self.packet[..], self.num);
        }
        copy.finalize64()
    }

    pub fn finish128(&self) -> [u64; 2] {
        let mut copy: HighwayHashState = self.state;
        if self.num != 0 {
            copy.update_remainder(&self.packet[..], self.num);
        }
        copy.finalize128()
    }

    pub fn finalize256(&self) -> [u64; 4] {
        let mut copy: HighwayHashState = self.state;
        if self.num != 0 {
            copy.update_remainder(&self.packet[..], self.num);
        }
        copy.finalize256()
    }
}
