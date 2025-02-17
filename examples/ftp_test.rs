extern crate ftp_file_sender;

use log::{error, info};
use env_logger;
use std::process;
use std::thread::sleep;
use std::time::Duration;

use ftp_file_sender::FtpSender;

fn main() {
    // ログの初期化
    env_logger::init();

    // FTP サーバの情報
    let host: &str = "127.0.0.1"; // FTP サーバのアドレス（ローカルテスト用）
    let port: u16 = 21; // FTP サーバのポート
    let timeout: f64 = 10.0; // タイムアウト（秒）
    let username: Option<&str> = Some("user"); // 必要なら FTP のユーザー名
    let password: Option<&str> = Some("password"); // 必要なら FTP のパスワード

    // アップロードするファイル
    let source_file: &str = "test_file.txt"; // 送信するファイル
    let target_folder: &str = "/uploads"; // サーバー側の保存先

    // テスト用のファイルを作成
    if let Err(e) = std::fs::write(source_file, "This is a test FTP file.") {
        error!("Failed to create test file: {}", e);
        process::exit(1);
    }

    // FtpSender のインスタンスを作成
    let ftp_sender: FtpSender = FtpSender::new(host, port, timeout, username, password);

    // ファイル送信
    match ftp_sender.send_file(source_file, target_folder) {
        Ok(_) => info!("Test file upload successful"),
        Err(e) => {
            error!("Test file upload failed: {}", e);
            process::exit(1);
        }
    }

    // 少し待ってから削除（FTP サーバ側に反映されるのを待つ）
    sleep(Duration::from_secs(5));

    // テストファイルの削除
    if let Err(e) = std::fs::remove_file(source_file) {
        error!("Failed to remove test file: {}", e);
    }
}
