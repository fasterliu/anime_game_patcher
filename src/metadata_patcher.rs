use regex::Regex;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

// For these two functions, a non-zero return value indicates failure.
extern "C" {
    fn decrypt_global_metadata(data: *mut u8, size: usize) -> i32;
    fn encrypt_global_metadata(data: *mut u8, size: usize) -> i32;
}

pub fn patch_metadata(metadata_file: &str) -> bool {
    println!("[1] Patching metadata file: {}", metadata_file);
    let decrypted = decrypt_metadata(metadata_file);
    if do_vecs_match(&decrypted, &Vec::new()) {
        println!("Failed to decrypt metadata file.");
        return false;
    }


    let decrypted_file = &(metadata_file.to_owned() + "-decrypted");
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .open(decrypted_file)
    {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open global-metadata-decrypted file: {}", e);
            return false;
        }
    };
    file.write_all(&decrypted).unwrap();


    let modified = replace_keys(&decrypted);
    if do_vecs_match(&modified, &Vec::new()) {
        println!("Failed to replace keys in metadata file.");
        return false;
    }

    let encrypted = encrypt_metadata(&modified);
    if do_vecs_match(&encrypted, &Vec::new()) {
        println!("Failed to re-encrypt metadata file.");
        return false;
    }

    //write encrypted to file
    let patched_file = &(metadata_file.to_owned() + "-patched");
    println!("[6] Write to patched metadata file: {}", patched_file);
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .open(patched_file)
    {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open global-metadata-patched file: {}", e);
            return false;
        }
    };
    file.write_all(&encrypted).unwrap();

    true
}

fn decrypt_metadata(file_path: &str) -> Vec<u8> {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open global-metadata: {}", e);
            return Vec::new();
        }
    };
    let mut data = Vec::new();

    // Read metadata file
    match file.read_to_end(&mut data) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to read global-metadata: {}", e);
            return Vec::new();
        }
    }

    // Decrypt metadata file
    let success = unsafe { decrypt_global_metadata(data.as_mut_ptr(), data.len()) } == 0;
    if success {
        println!("[2] Successfully decrypted global-metadata");
        data
    } else {
        println!("Failed to decrypt global-metadata");
        Vec::new()
    }
}

fn replace_keys(data: &[u8]) -> Vec<u8> {
    let mut new_data = String::new();

    unsafe {
        let data_str = String::from_utf8_unchecked(data.to_vec());

        let re = Regex::new(r"<RSAKeyValue>((.|\n|\r|\t| )*?)</RSAKeyValue>").unwrap();
        let matches = re.find_iter(&data_str);

        println!("==================<DEBUG>==================");
        for (i, rmatch) in matches.enumerate() {
            let key = rmatch.as_str();
            println!("Key index = {} ({} bytes)\n{}\n----------", i, key.len(), key);
        }
        println!("==================<DEBUG>==================");

        // dispatch key is index 3
        // password key is index 2

        let matches = re.find_iter(&data_str);

        for (i, rmatch) in matches.enumerate() {
            let key = rmatch.as_str();

            if i == 2 {
                println!("[3] Replacing password key: {}", i);
                new_data = replace_rsa_key(&data_str, key, "passwordKey.txt");
            } else if i == 3 {
                println!("[4] Replacing dispatch key: {}", i);
                new_data = replace_rsa_key(&new_data, key, "dispatchKey.txt");
            }
        }
    }

    return new_data.as_bytes().to_vec();
}

fn replace_rsa_key(old_data: &str, to_replace: &str, file_name: &str) -> String {
    // Read dispatch key file
    unsafe {
        // Get key folder from exe path
        let mut exe_path = std::env::current_exe().unwrap();
        exe_path.pop();

        let key_folder = exe_path.to_str().unwrap().to_string();
        let mut new_key_file = match File::open(format!("{}/keys/{}", key_folder, file_name)) {
            Ok(file) => file,
            Err(e) => {
                println!("Failed to open keys/{}: {}", file_name, e);
                return String::new();
            }
        };
        let mut key_data = Vec::new();
        new_key_file.read_to_end(&mut key_data).unwrap();
        let new_key = String::from_utf8_unchecked(key_data.to_vec());

        // Replace old key with new key
        if to_replace.len() != new_key.len() {
            println!("[DEBUG] Replacement size mismatch:\nOriginal: {} bytes------\n{}\nReplacement: {} bytes------\n{}",
                     to_replace.len(), to_replace, new_key.len(), new_key);
        }
        old_data.replace(to_replace, &new_key)
    }
}

fn encrypt_metadata(old_data: &[u8]) -> Vec<u8> {
    let mut data = old_data.to_vec();
    let success = unsafe { encrypt_global_metadata(data.as_mut_ptr(), data.len()) } == 0;
    if success {
        println!("[5] Successfully encrypted global-metadata");
        data
    } else {
        println!("Failed to encrypt global-metadata");
        Vec::new()
    }
}

fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    a == b
}
