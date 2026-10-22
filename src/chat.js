"use strict";//厳格モードで実装

/** 
 * id=your_containerに対してチャットUIを組み込む 
 **/
document.addEventListener("DOMContentLoaded", function() {//htmlの描画が終わったら、処理を実行する記述

    //文字列(html要素)を格納できる配列を定義
    var chatDom =[];
    //文字列（この場合は引数のdom）を配列の要素として格納できる関数
    var p = function(dom){
        chatDom.push(dom);
    };

    //配列の要素として一つ一つ格納していき、最後にjoin関数で結合する
    //チャットの外側部分①
    p('<div id="bms_messages_container">');

        //ヘッダー部分②
        p('<div id="bms_chat_header">');
            p('<div id="bms_chat_user_status">');
                p('<div id="bms_status_icon">●</div>')
                p('<div id="bms_chat_user_name">ユーザー</div>');
            p('</div>');
        p('</div>');
              //タイムライン部分③
        p('<div id="bms_messages">');

            
        p('</div>');

        // テキストボックス、送信ボタン④
        p('<div id="bms_send">');
            p('<textarea id="bms_send_message"></textarea>');
            p('<div id="bms_send_btn">送信</div>');
        p('</div>');

        //jQueryのappend関数を使って、your_containerの直下にhtml要素を書き加える
        document.querySelector('#your_container').insertAdjacentHTML('beforeend',chatDom.join(''));
        // UIができてから送受信の準備をする
        setupChat();
});

// ========== P2P送受信 ==========

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// 送信中に二重送信しないためのフラグ
let sending = false;

// 吹き出しを1つ追加する
function addMessage(text, isMine) {
    const messages = document.querySelector('#bms_messages');

    const message = document.createElement("div")
    message.className = "bms_message " + (isMine ? "bms_right" : "bms_left");

    const box = document.createElement("div");
    box.className = "bms_message_box";

    const content = document.createElement("div");
    content.className = "bms_message_content";

    const textEl = document.createElement("div");
    textEl.className = "bms_message_text";
    // textContentで入れる（相手の本文をHTMLとして解釈させない）
    textEl.textContent = text;

    content.appendChild(textEl);
    box.appendChild(content);
    message.appendChild(box);
    
    const clear = document.createElement("div");
    clear.className = "bms_clear";

    messages.appendChild(message);
    messages.appendChild(clear);

    // 一番下までスクロール
    messages.scrollTop = messages.scrollHeight;
}

// 送信ボタン or Enterで呼ばれる
async function sendMEssage() {
    if (sending) return;

    const textarea = document.querySelector("#bms_send_message");
    const partnerId = document.querySelector("#partner_id").value.trim();
    const text = textarea.value.trim();

    if (!partnerId) {
        alert("相手のIDを入力してください");
        return;
    }
    if (!text) return;

    sending = true;
    try {
        // Rustのsend_messageを呼ぶ。相手からACKが帰るまで待つ
        await invoke("send_message", {to: partnerId, text: text});
        addMessage(text, true);
        textarea.value = "";
    } catch (e) {
        alert("送信失敗: " + e);
    } finally {
        sending = false;
    }
}

async function setupChat() {
    // 自分のIDを表示
    const myId = await invoke("my_id");
    document.querySelector("#my_id").value = myId;

    // 送信ボタン
    document.querySelector("#bms_send_btn").addEventListener("click", sendMEssage);

    // Enterで送信、Shift+Enterで改行（日本語変換中のEnterは無視）
    document.querySelector("#bms_send_message").addEventListener("keydown", (e) => {
        if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
            e.preventDefault();
            sendMEssage();
        }
    });

    // Rustから "chat-message" イベントが来たら左側に表示
    await listen("chat-message", (event) => {
        addMessage(event.payload.text, false);
        // ヘッダーに送信者IDの先頭8文字を出す
        document.querySelector("#bms_chat_user_name").
            textContent = event.payload.from.slice(0, 8);
    })
}