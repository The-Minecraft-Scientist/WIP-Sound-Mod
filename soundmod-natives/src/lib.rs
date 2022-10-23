extern crate libc;

use libc::size_t;

#[no_mangle]
unsafe extern "C" fn test_fn1() -> f32 {
    println!("hello from rust!");
    return 3.0 as f32
}

#[no_mangle]
unsafe extern "C" fn test_fn2(pointer: size_t, size: i32) {
    let array = unsafe {std::slice::from_raw_parts(pointer as *const i32, size as usize)};
}
