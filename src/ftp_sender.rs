use ftp::FtpStream;
use log::{error, info};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
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
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
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
            let err_msg = format!(
                "Source file '{}' does not exist or is not a file.",
                source_file_path
            );
            error!("{}", err_msg);
            return Err(err_msg.into());
        }

        // 送信するファイル名を取得
        let filename = source_path
            .file_name()
            .ok_or("Failed to get the source file name")?;

        // FTP サーバへの接続
        let addr = format!("{}:{}", self.host, self.port);
        let mut ftp_stream = FtpStream::connect(addr)?;
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

        // リモートの target_folder に移動。存在しなければ作成してから移動
        self.ensure_remote_folder_exists(&mut ftp_stream, target_folder)?;

        // ファイル送信
        info!(
            "Sending file '{}' to folder '{}' on the server",
            source_file_path, target_folder
        );
        let mut file = File::open(&source_path)?;
        // 現在の作業ディレクトリが target_folder に切り替わっているため、ファイル名のみ指定
        let filename_str = filename.to_string_lossy();
        ftp_stream.put(&filename_str, &mut file)?;
        info!(
            "File '{}' sent successfully to folder '{}'",
            source_file_path, target_folder
        );

        // 接続終了
        ftp_stream.quit()?;
        info!("FTP connection closed");

        Ok(())
    }

    /// リモートフォルダーが存在するか確認し、存在しなければ作成して移動する
    fn ensure_remote_folder_exists(
        &self,
        ftp_stream: &mut FtpStream,
        folder: &str,
    ) -> Result<(), Box<dyn Error>> {
        match ftp_stream.cwd(folder) {
            Ok(_) => {
                info!("Remote folder '{}' exists. Switched to it.", folder);
            }
            Err(_) => {
                info!("Remote folder '{}' does not exist. Creating it...", folder);
                ftp_stream.mkdir(folder)?;
                ftp_stream.cwd(folder)?;
                info!("Remote folder '{}' created and switched to.", folder);
            }
        }
        Ok(())
    }
}
