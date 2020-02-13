use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn encrypt_and_decrypt() {
    let temp = assert_fs::TempDir::new().unwrap();

    let password = "plop";
    let salt = "12345678901234567890123456789012";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";
    let chunk_size = "512"; //force multiple pass

    let original = "tests/fixtures/computer.svg";
    let encrypted = temp.child("computer.svg.enc");
    let decrypted = temp.child("computer.dec.svg");

    let encrypted_path = encrypted.path();
    let decrypted_path = decrypted.path();

    let mut encrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    encrypt_cmd
        .arg("encrypt")
        .arg(original)
        .arg(encrypted_path)
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password)
        .env("DS_SALT", salt)
        .env("DS_CHUNK_SIZE", chunk_size);

    encrypt_cmd.assert().success();

    let mut decrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    decrypt_cmd
        .arg("decrypt")
        .arg(encrypted_path)
        .arg(decrypted_path)
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password)
        .env("DS_SALT", salt)
        .env("DS_CHUNK_SIZE", chunk_size);

    decrypt_cmd.assert().success();

    let original_bytes = std::fs::read(original).unwrap();
    let decrypted_bytes = std::fs::read(decrypted_path).unwrap();

    temp.close().unwrap();

    assert_eq!(original_bytes, decrypted_bytes);
}

#[test]
fn decrypting_a_plaintext_file_yields_the_original_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    let password = "plop";
    let salt = "12345678901234567890123456789012";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";

    let original = "tests/fixtures/computer.svg";
    let encrypted = "tests/fixtures/computer.svg.enc";
    let decrypted = temp.child("computer.dec.svg");
    let decrypted_path = decrypted.path();

    let mut decrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    decrypt_cmd
        .arg("decrypt")
        .arg(encrypted)
        .arg(decrypted_path)
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password)
        .env("DS_SALT", salt);

    decrypt_cmd.assert().success();

    let original_bytes = std::fs::read(original).unwrap();
    let decrypted_bytes = std::fs::read(decrypted_path).unwrap();

    temp.close().unwrap();

    assert_eq!(original_bytes, decrypted_bytes);
}

#[test]
fn the_app_crashes_on_a_missing_password() {
    let temp = assert_fs::TempDir::new().unwrap();

    let salt = "12345678901234567890123456789012";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";

    let encrypted = "tests/fixtures/computer.svg.enc";
    let decrypted = temp.child("computer.dec.svg");
    let decrypted_path = decrypted.path();

    let mut decrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    decrypt_cmd
        .arg("proxy")
        .arg(encrypted)
        .arg(decrypted_path)
        .arg(hash_file_arg)
        .env("DS_SALT", salt);

    decrypt_cmd.assert().failure();
}

#[test]
fn the_app_crashes_on_a_missing_hash() {
    let temp = assert_fs::TempDir::new().unwrap();

    let password = "plop";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";

    let encrypted = "tests/fixtures/computer.svg.enc";
    let decrypted = temp.child("computer.dec.svg");
    let decrypted_path = decrypted.path();

    let mut decrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    decrypt_cmd
        .arg("proxy")
        .arg(encrypted)
        .arg(decrypted_path)
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password);

    decrypt_cmd.assert().failure();
}

#[test]
fn the_app_crashes_with_an_invalid_password() {
    let temp = assert_fs::TempDir::new().unwrap();

    let password = "this is not the expected password";
    let salt = "12345678901234567890123456789012";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";

    let encrypted = "tests/fixtures/computer.svg.enc";
    let decrypted = temp.child("computer.dec.svg");
    let decrypted_path = decrypted.path();

    let mut decrypt_cmd = Command::cargo_bin("ds_proxy").unwrap();
    decrypt_cmd
        .arg("proxy")
        .arg(encrypted)
        .arg(decrypted_path)
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password)
        .env("DS_SALT", salt);

    decrypt_cmd.assert().failure();
}

use std::{thread, time};


#[test]
fn end_to_end_upload_and_download() {
    use std::path::Path;
    use encrypt::header::{PREFIX, PREFIX_SIZE};
    /*
    This test:
     - spawns a node server that stores uploaded files in tests/fixtures/server-static/uploads/
     - spawns a ds proxy that uses the node proxy as a storage backend
     - uploads a file using curl via the DS proxy
     - checks that said file is encrypted
     - downloads the encrypted file via the proxy, and checks that its content matches the initial content
    */
    let password = "plop";
    let salt = "12345678901234567890123456789012";
    let hash_file_arg = "--hash-file=tests/fixtures/password.hash";
    let chunk_size = "512"; //force multiple pass

    let original = "tests/fixtures/computer.svg";
    let encrypted_file = "tests/fixtures/server-static/uploads/victory";

    if Path::new(encrypted_file).exists() {
        std::fs::remove_file(encrypted_file)
            .expect(&format!("Unable to remove {} !", encrypted_file.to_owned()));
    }

    let mut proxy_server_command = Command::cargo_bin("ds_proxy").unwrap();
    let mut proxy_server = proxy_server_command
        .arg("proxy")
        .arg("--address=localhost:4444")
        .arg("--upstream-url=http://localhost:3000")
        .arg(hash_file_arg)
        .env("DS_PASSWORD", password)
        .env("DS_SALT", salt)
        .env("DS_CHUNK_SIZE", chunk_size)
        .spawn()
        .unwrap();

    let mut node_server = Command::new("node")
        .arg("tests/fixtures/server-static/server.js")
        .spawn()
        .expect("failed to execute child");

    thread::sleep(time::Duration::from_millis(5000));
    println!("on envoie la purée !");

    let curl_upload = Command::new("curl")
        .arg("-XPUT")
        .arg("localhost:4444/victory")
        .arg("--data-binary")
        .arg("@tests/fixtures/computer.svg")
        .output()
        .expect("failed to perform upload");

    println!("{:?}", String::from_utf8_lossy(&curl_upload.stdout));
    println!("{:?}", String::from_utf8_lossy(&curl_upload.stderr));

    if curl_upload.status.success() {
        let encrypted_bytes = std::fs::read(encrypted_file).expect("encrypted should exist !");
        assert_eq!(&encrypted_bytes[0..PREFIX_SIZE], PREFIX);

        let curl_download = Command::new("curl")
            .arg("-XGET")
            .arg("localhost:4444/victory")
            .output()
            .expect("failed to perform download");

        let original_bytes = std::fs::read(original).expect("original should exist !");
        assert_eq!(curl_download.stdout, original_bytes);
    } else {
        panic!("unable to check file upload !");
    }

    proxy_server.kill().expect("killing the proxy server should succeed !");
    node_server.kill().expect("killing node's upload server should succeed !");
}
