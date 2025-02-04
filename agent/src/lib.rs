use tokio;

pub mod cli;

pub async fn run(controlled: bool, options: &cli::AgentOptions) -> std::io::Result<()> {
    println!("Starting MiaX Agent...");

    // 後ほどAxumサーバーを起動する処理を実装
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000 !!");

    // ここに追加のサーバー設定やルーティングを実装予定
    Ok(())
}
