use clap::Parser;
use log::error;
use ftp_file_sender::FtpSender;
use ftp_file_sender::setup_logger;

/// FTP File Sender
///
/// FTP を使って指定したファイルをサーバに送信します。
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

    // FtpSender インスタンスを作成
    let ftp_sender: FtpSender = FtpSender::new(
        &args.host,
        args.port,
        args.timeout,
        args.username.as_deref(),
        args.password.as_deref(),
    );
    
    // ファイル送信の実行
    if let Err(e) = ftp_sender.send_file(&args.path, &args.folder) {
        error!("Failed to send file: {}", e);
        eprintln!("Failed to send file: {}", e);
        std::process::exit(1);
    }
}
