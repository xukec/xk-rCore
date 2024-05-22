const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe {
                //接受一个指向某个内存区域的原始指针buf和一个长度len，并返回一个指向这个内存区域的切片。
                core::slice::from_raw_parts(buf, len)
            };
            //它接受一个字节切片，并尝试将其解释为一个UTF-8字符串
            let str = core::str::from_utf8(slice).unwrap(); 
            print!("{}", str);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}