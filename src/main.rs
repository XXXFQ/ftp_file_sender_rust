mod logger;

use clap::Parser;
use ftp::FtpStream;
use log::{error, info};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::logger::setup_logger;

/// FTP File Sender
///
/// このプログラムは、FTP を使って指定したファイルをサーバに送信します。
#[derive(Parser, Debug)]
#[command(author, version, about = "FTP File Sender", long_about = None)]
struct Args {
    /// FTP サーバのホストアドレス (例: 192.168.1.3)
    host: String,

    /// 送信するファイルのパス (例: ./example_file.txt)
    path: String,

    /// サーバ上の転送先フォルダ
    #[arg(short = 'f', long = "folder", default_value = "./")]
    folder: String,

    /// 接続に使用するポート番号
    #[arg(short = 'p', long = "port", default_value_t = 20)]
    port: u16,

    /// 接続タイムアウト (秒)
    #[arg(short = 't', long = "timeout", default_value_t = 30.0)]
    timeout: f64,

    /// FTP ログイン用のユーザー名
    #[arg(short = 'u', long = "username")]
    username: Option<String>,

    /// FTP ログイン用のパスワード
    #[arg(short = 'w', long = "password")]
    password: Option<String>,
}

/// FTP を使ってファイルを送信する関数
fn send_file_over_ftp(
    host: &str,
    source_file_path: &str,
    target_folder: &str,
    port: u16,
    timeout_secs: f64,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Attempting to send file via FTP...");

    // ファイルパスを正規化してディレクトリトラバーサル攻撃対策
    let source_path: PathBuf = std::fs::canonicalize(source_file_path)?;
    if !source_path.is_file() {
        let err_msg: String = format!(
            "Source file '{}' does not exist or is not a file.",
            source_file_path
        );
        error!("{}", err_msg);
        return Err(err_msg.into());
    }

    // サーバ上の転送先ファイルパス: target_folder + ファイル名
    let filename: &std::ffi::OsStr = source_path
        .file_name()
        .ok_or("Failed to get the source file name")?;
    let target_file_path = Path::new(target_folder).join(filename);
    let target_file_path_str = target_file_path.to_string_lossy();

    // FTP サーバへの接続
    let addr: String = format!("{}:{}", host, port);
    let timeout: Duration = Duration::from_secs_f64(timeout_secs);
    let mut ftp_stream: FtpStream = FtpStream::connect(addr)?;
    // タイムアウト設定
    ftp_stream.get_ref().set_read_timeout(Some(timeout))?;
    ftp_stream.get_ref().set_write_timeout(Some(timeout))?;
    info!("Connected to {} on port {}", host, port);

    // サーバからのウェルカムメッセージ（取得に失敗しても無視）
    match ftp_stream.read_response(220) {
        Ok(response) => info!("Server response: {}", response.1),
        Err(e) => info!("Could not get welcome message: {}", e),
    };

    // ログイン処理：ユーザー名とパスワードが指定されていればそれを使用、なければ匿名ログイン
    if let (Some(user), Some(pass)) = (username, password) {
        ftp_stream.login(user, pass)?;
        info!("Login successful with provided credentials");
    } else {
        ftp_stream.login("anonymous", "anonymous")?;
        info!("Anonymous login successful");
    }

    info!(
        "Sending file '{}' to '{}' on the server",
        source_file_path, target_folder
    );
    // ファイルを読み込み、FTP サーバへ送信
    let mut file: File = File::open(&source_path)?;
    ftp_stream.put(&target_file_path_str, &mut file)?;
    info!(
        "File '{}' sent successfully to '{}'",
        source_file_path, target_folder
    );

    ftp_stream.quit()?;
    info!("FTP connection closed");

    Ok(())
}

fn main() {
    // ロガーを初期化
    if let Err(e) = setup_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

    let args: Args = Args::parse();

    // 引数チェック：ホストとファイルパスは必須
    if args.host.trim().is_empty() {
        error!("Host address is required");
        eprintln!("Host address is required");
        std::process::exit(1);
    }
    if args.path.trim().is_empty() {
        error!("File path is required");
        eprintln!("File path is required");
        std::process::exit(1);
    }

    // ファイル送信の実行
    if let Err(e) = send_file_over_ftp(
        &args.host,
        &args.path,
        &args.folder,
        args.port,
        args.timeout,
        args.username.as_deref(),
        args.password.as_deref(),
    ) {
        error!("FTP error occurred: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
