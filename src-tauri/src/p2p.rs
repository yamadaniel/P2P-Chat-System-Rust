use iroh::{
    endpoint::{presets, Connection},
    protocol::{AcceptError, ProtocolHandler, Router},
    Endpoint, EndpointId,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

// LynQ専用のプロトコル名（送信側と受信側で一致させる）
pub const ALPN: &[u8] = b"lynq/chat/0";

// manageに置くもの: EndpointとRouter(電話機と受信係)
pub struct P2p {
    pub endpoint: Endpoint,
    pub router: Router,
}

// 起動時に1回だけ呼ぶ。Endpointを用意して、Routerを待機させる
pub async fn start(app: AppHandle) -> anyhow::Result<P2p> {
    // TODO: 今は起動のたびにIDが変わる。後で秘密鍵を保存、読込する形にする
    let endpoint = Endpoint::bind(presets::N0).await?;

    // Routerに渡すと所有権が移るので、Coneを渡して手元にも残す
    let router = Router::builder(endpoint.clone())
        .accept(ALPN, ChatHandler { app})
        .spawn();

    Ok(P2p { endpoint, router})

}

// ========== 受信 ==========

// フロントに送る受信メッセージの形
#[derive(Clone, Serialize)]
struct RecivedMessage {
    from: String,
    text: String,
}

// Echo代わりのRouter。画面に知らせるためにAppHandleを持つ
#[derive(Debug, Clone)]
struct ChatHandler {
    app: AppHandle
}

impl ProtocolHandler for ChatHandler {
    async fn accept(&self, connection: Connection) -> Result <(), AcceptError> {
        // 送信者のID(irohがカギで認証済み)
        let from = connection.remote_id();

        let (mut send, mut recv) = connection.accept_bi().await?;

        // 本文を受け取る（最大64KB）
        let bytes = recv
            .read_to_end(64*1024)
            .await
            .map_err(AcceptError::from_err)?;
        let text = String::from_utf8_lossy(&bytes).to_string();

        // 画面(JS)に"chat-message"イベントで知らせる
        let _ = self.app.emit(
            "chat-message",
            RecivedMessage {
                from: from.to_string(),
                text,
            },
        );

        // 受け取ったことを送信者に返す（ACK)
        send.write_all(b"ok").await.map_err(AcceptError::from_err)?;
        send.finish()?;

        connection.closed().await;
        Ok(())
    }
}

// ========= JSから呼ぶコマンド ==========

// 自分のIDを返す
#[tauri::command]
pub fn my_id(p2p: State<'_, P2p>) -> String {
    p2p.endpoint.id().to_string()
}

// 相手のIDに本文を送る。相手からACKが帰ってくれば成功
#[tauri::command]
pub async fn send_message(p2p: State<'_, P2p>, to: String, text: String) -> Result <(), String> {
    let to: EndpointId = to.parse().map_err(err)?;

    // manageのEndpointを使う（新しく作らない）
    let conn = p2p.endpoint.connect(to, ALPN).await.map_err(err)?;
    let (mut send, mut recv) = conn.open_bi().await.map_err(err)?;

    send.write_all(text.as_bytes()).await.map_err(err)?;
    send.finish().map_err(err)?;

    let ack = recv.read_to_end(16).await.map_err(err)?;
    conn.close(0u32.into(), b"done");

    if ack == b"ok" {
        Ok(())
    } else {
        Err("相手から受信確認が帰らなかった". into())
    }

}

// エラーを文字列にしてJSへ返すための補助
fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}