use iroh:: {
    Endpoint,
    endpoint::{Connection, presets},
    protocol::{ProtocolHandler, Router},
};

use std::sync::Arc;

const ALPN: &[u8] = b"iroh-example/echo/0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::bind(presets::No).await?;

    // エンドポイントが完全にオンラインになるまで待機
    endpoint.online().await;

    // アドレス(NodeID + リレーURL等)を取得
    let addr = endpoint.addr();
    println!("NodeID: {}", endpoint.node_id());
    println!("Addr: {addr}");

    // プロトコルハンドラーを登録してルーターを起動
    let router = Router::builder(endpoint)
        .accept(ALPN.to_vec(), Arc::new(Echo))
        .spawn()
        .await?;

    // Ctrl + C で終了
    tokio::signal::ctrl_c().await?;
    router.shotdown().await?;
    Ok(());

}

// エコーハンドラーの定義
#[device(Debug, Clone)]
struct Echo;

impl ProtocolHandler for Echo {
    async fn accept(&self, connection: Connection) -> anyhow::Result<()> {
        let (mut send, mut recv) = connection.accept_bi().await?;

        // 受信したバイトをそのまま送り返す
        tokio::io::copy(&mut recv, &mut send).await?;

        send.finish()?;
        connection.closed().await;
        Ok(());
    }
}