
use critical_section;


const BASE_ADDRESS: usize = 0x1FF80050;
const OFFSETS: [usize; 3] = [0x00, 0x04, 0x14];

/// Read a 32-bit value from the given address.
fn read_u32_at_address(address: *const u32) -> u32 {
    unsafe { address.read_volatile() }
}

/// Get the unique 96-bit ID as an array of 12 bytes.
fn uid() -> [u8; 12] {
    let mut uid_bytes = [0u8; 12];
    let uids = [
        read_u32_at_address((BASE_ADDRESS + OFFSETS[0]) as *const u32),
        read_u32_at_address((BASE_ADDRESS + OFFSETS[1]) as *const u32),
        read_u32_at_address((BASE_ADDRESS + OFFSETS[2]) as *const u32),
    ];

    // Convert each u32 to bytes and store them in uid_bytes
    for (i, &word) in uids.iter().enumerate() {
        let word = word.to_be(); // Convert to big-endian
        uid_bytes[i * 4] = (word >> 24) as u8;
        uid_bytes[i * 4 + 1] = (word >> 16) as u8;
        uid_bytes[i * 4 + 2] = (word >> 8) as u8;
        uid_bytes[i * 4 + 3] = word as u8;
    }

    uid_bytes
}

/// Get this device's unique 96-bit ID, encoded into a string of 24 hexadecimal ASCII digits.
pub fn uid_hex() -> &'static str {
    unsafe { core::str::from_utf8_unchecked(uid_hex_bytes()) }
}

/// Get this device's unique 96-bit ID, encoded into 24 hexadecimal ASCII bytes.
pub fn uid_hex_bytes() -> &'static [u8; 24] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    static mut UID_HEX: [u8; 24] = [0; 24];
    static mut LOADED: bool = false;
    critical_section::with(|_| unsafe {
        if !LOADED {
            let uid = uid();
            for (idx, v) in uid.iter().enumerate() {
                let lo = v & 0x0f;
                let hi = (v & 0xf0) >> 4;
                UID_HEX[idx * 2] = HEX[hi as usize];
                UID_HEX[idx * 2 + 1] = HEX[lo as usize];
            }
            LOADED = true;
        }
    });
    unsafe { &*core::ptr::addr_of!(UID_HEX) }
}
