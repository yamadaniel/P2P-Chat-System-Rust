use iroh::{Endpoint, EndpointId, endpoint::presets};

const ALPN: &[u8] = b"iroh-example/echo/0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 接続先のアドレスを引数から取得
    let addr: EndpointId = std::env::args().nth(1)
    .expect("Usage: client <addr>")
    .parse()?;

    // エンドポイントを作成してバインド
    let endpoint = Endpoint::bind(presets::N0).await?;

    // 接続先に接続(ALPN指定)
    let conn = endpoint.connect(addr, ALPN).await?;

    // 双方向QUICストリームを開く
    let (mut send, mut recv) = conn.open_bi().await?;

    // データを送信
    send.write_all(b"Hello, Iroh!").await?;
    send.finish()?;

    // エコーを受信
    let response = recv.read_to_end(1000).await?;
    println!("受信: {}", String::from_utf8_lossy(&response));

    // 接続を閉じる
    conn.close(0u32.into(), b"bye!");
    endpoint.close().await;
    Ok(())
}