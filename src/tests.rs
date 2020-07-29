use super::Volatile;

#[test]
fn test_read() {
    assert_eq!(Volatile::new(42).read(), 42);
}

#[test]
fn test_write() {
    let mut volatile = Volatile::new(42);
    volatile.write(50);
    assert_eq!(volatile.value, 50);
}

#[test]
fn test_update() {
    let mut volatile = Volatile::new(42);
    volatile.update(|v| *v += 1);
    assert_eq!(volatile.value, 43);
}

#[test]
fn test_pointer_recast() {
    let mut target_value = 0u32;

    let target_ptr: *mut u32 = &mut target_value;
    let volatile_ptr = target_ptr as *mut Volatile<u32>;

    // UNSAFE: Safe, as we know the value exists on the stack.
    unsafe {
        (*volatile_ptr).write(42u32);
    }

    assert_eq!(target_value, 42u32);
}
