use {
    std::{convert::TryFrom, env::args, slice},
    winapi::{
        ctypes::c_void,
        um::{
            dpapi::{CryptUnprotectData, CRYPTPROTECT_PROMPTSTRUCT},
            winbase::LocalFree,
            wincrypt::CRYPTOAPI_BLOB,
        },
    },
};

fn main() {
    let mut plaintext: Vec<u8> = "hello".to_string().into_bytes();

    let mut input = CRYPTOAPI_BLOB {
        cbData: u32::try_from(plaintext.len()).unwrap(),
        pbData: plaintext.as_mut_ptr(),
    };

    let cleartext: Result<Vec<u8>, ()>;

    unsafe {
        let mut output = CRYPTOAPI_BLOB::default();

        if CryptUnprotectData(
            &mut input,
            0 as *mut *mut u16,
            0 as *mut CRYPTOAPI_BLOB,
            0 as *mut c_void,
            0 as *mut CRYPTPROTECT_PROMPTSTRUCT,
            0 as u32,
            &mut output,
        ) != 0
        {
            cleartext = Ok(slice::from_raw_parts(
                output.pbData,
                usize::try_from(output.cbData).unwrap(),
            )
            .iter()
            .cloned()
            .collect());
        } else {
            cleartext = Err(())
        }

        LocalFree(output.pbData as *mut c_void);
    }

    print!("{:?}", cleartext);
}
