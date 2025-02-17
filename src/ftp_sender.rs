use ftp::FtpStream;
use log::{error, info};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// FTP 送信機能を提供する構造体
pub struct FtpSender {
    host: String,
    port: u16,
    timeout: Duration,
    username: Option<String>,
    password: Option<String>,
}

impl FtpSender {
    /// 新しい FtpSender を作成する
    pub fn new(
        host: &str,
        port: u16,
        timeout_secs: f64,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Self {
        FtpSender {
            host: host.to_string(),
            port,
            timeout: Duration::from_secs_f64(timeout_secs),
            username: username.map(|s: &str| s.to_string()),
            password: password.map(|s: &str| s.to_string()),
        }
    }

    /// FTP サーバーに接続し、ログイン後、指定されたファイルを指定のフォルダーに送信する
    pub fn send_file(
        &self,
        source_file_path: &str,
        target_folder: &str,
    ) -> Result<(), Box<dyn Error>> {
        info!("Attempting to send file via FTP...");

        // ファイルパスを正規化して、ディレクトリトラバーサル攻撃対策
        let source_path: PathBuf = std::fs::canonicalize(source_file_path)?;
        if !source_path.is_file() {
            let err_msg: String = format!(
                "Source file '{}' does not exist or is not a file.",
                source_file_path
            );
            error!("{}", err_msg);
            return Err(err_msg.into());
        }

        // 送信するファイル名を取得
        let filename: &std::ffi::OsStr = source_path
            .file_name()
            .ok_or("Failed to get the source file name")?;
        let target_file_path = Path::new(target_folder).join(filename);
        let target_file_path_str = target_file_path.to_string_lossy();

        // FTP サーバへの接続
        let addr: String = format!("{}:{}", self.host, self.port);
        let mut ftp_stream: FtpStream = FtpStream::connect(addr)?;
        ftp_stream.get_ref().set_read_timeout(Some(self.timeout))?;
        ftp_stream.get_ref().set_write_timeout(Some(self.timeout))?;
        info!("Connected to {} on port {}", self.host, self.port);

        // ログイン処理
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => {
                ftp_stream.login(user, pass)?;
                info!("Login successful with provided credentials");
            }
            _ => {
                ftp_stream.login("anonymous", "anonymous")?;
                info!("Anonymous login successful");
            }
        }

        // ファイル送信
        info!(
            "Sending file '{}' to folder '{}' on the server",
            source_file_path, target_folder
        );
        let mut file: File = File::open(&source_path)?;
        ftp_stream.put(&target_file_path_str, &mut file)?;
        info!(
            "File '{}' sent successfully to folder '{}'",
            source_file_path, target_folder
        );

        // 接続終了
        ftp_stream.quit()?;
        info!("FTP connection closed");

        Ok(())
    }
}
