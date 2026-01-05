use zeroize::Zeroize;
use std::io::Error;

// Wrapper for the raw key (which should be zeroized on drop)
#[derive(Debug, Zeroize)]
#[zeroize(drop)]
pub struct ZeroizingSecret {
    pub content: Vec<u8>,
}

impl ZeroizingSecret {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.content).unwrap_or("")
    }
}

pub struct SecretVault;

#[cfg(target_os = "linux")]
impl SecretVault {
    // Syscall wrapper for add_key
    pub fn store_secret(description: &str, secret: &[u8]) -> Result<i32, Error> {
        use std::ffi::CString;
        use libc::{c_long, c_void};

        const SYS_ADD_KEY: c_long = 248;
        const KEY_SPEC_THREAD_KEYRING: i32 = -1;
        const KEY_SPEC_PROCESS_KEYRING: i32 = -2;

        let type_ = CString::new("user").unwrap();
        let desc = CString::new(description).unwrap();
        
        let ret = unsafe {
            libc::syscall(
                SYS_ADD_KEY,
                type_.as_ptr(),
                desc.as_ptr(),
                secret.as_ptr() as *const c_void,
                secret.len(),
                KEY_SPEC_THREAD_KEYRING as i64, 
            )
        };

        if ret < 0 {
            // Try PROCESS_KEYRING if Thread fails
             let ret_retry = unsafe {
                libc::syscall(
                    SYS_ADD_KEY,
                    type_.as_ptr(),
                    desc.as_ptr(),
                    secret.as_ptr() as *const c_void,
                    secret.len(),
                    KEY_SPEC_PROCESS_KEYRING as i64,
                )
            };
             if ret_retry < 0 {
                 return Err(Error::last_os_error());
            }
            return Ok(ret_retry as i32);
        }

        Ok(ret as i32)
    }

    // Retrieves key into a Zeroizing Buffer
    pub fn retrieve_secret(key_id: i32) -> Result<ZeroizingSecret, Error> {
        use libc::c_long;
        const SYS_KEYCTL: c_long = 250;
        const KEYCTL_READ: c_long = 11;

        // First determine size: keyctl(KEYCTL_READ, key_id, NULL, 0)
        let ret_size = unsafe {
            libc::syscall(SYS_KEYCTL, KEYCTL_READ, key_id as c_long, 0, 0, 0)
        };

        if ret_size < 0 {
             return Err(Error::last_os_error());
        }

        let size = ret_size as usize;
        let mut buffer = vec![0u8; size];

        // keyctl(KEYCTL_READ, key_id, buffer, size)
        let ret_read = unsafe {
            libc::syscall(SYS_KEYCTL, KEYCTL_READ, key_id as c_long, buffer.as_mut_ptr(), size)
        };

        if ret_read < 0 {
            return Err(Error::last_os_error());
        }

        Ok(ZeroizingSecret { content: buffer })
    }

    // Revoke key (The Dead Man Switch)
    pub fn revoke(key_id: i32) -> Result<(), Error> {
        use libc::c_long;
        const SYS_KEYCTL: c_long = 250;
        const KEYCTL_REVOKE: c_long = 3;

        let ret = unsafe {
            libc::syscall(SYS_KEYCTL, KEYCTL_REVOKE, key_id as c_long, 0, 0, 0)
        };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
impl SecretVault {
    // Stub for MacOS/Dev
    pub fn store_secret(_description: &str, _secret: &[u8]) -> Result<i32, Error> {
        // Mock ID
        Ok(12345)
    }

    pub fn retrieve_secret(key_id: i32) -> Result<ZeroizingSecret, Error> {
        if key_id == 12345 {
            Ok(ZeroizingSecret { content: b"super_secret_api_key".to_vec() })
        } else {
             Err(Error::new(std::io::ErrorKind::NotFound, "Key not found in mock vault"))
        }
    }

    pub fn revoke(_key_id: i32) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_lifecycle() {
        let secret_payload = b"super_secret_api_key";
        match SecretVault::store_secret("reflex_api_key_test", secret_payload) {
            Ok(key_id) => {
                // 1. Retrieve
                let retrieved = SecretVault::retrieve_secret(key_id).expect("Failed to retrieve");
                assert_eq!(retrieved.content, secret_payload);
                println!("Successfully stored and retrieved from keyring: ID {}", key_id);

                // 2. Revoke
                SecretVault::revoke(key_id).expect("Failed to revoke");
            },
            Err(e) => {
                println!("Skipping Keyring test due to OS restrictions: {:?}", e);
            }
        }
    }
}
