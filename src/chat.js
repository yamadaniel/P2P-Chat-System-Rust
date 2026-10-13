"use strict";//厳格モードで実装

/** 
 * id=your_containerに対してチャットUIを組み込む 
 **/
$(document).ready(function() {//htmlの描画が終わったら、処理を実行する記述

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

            //メッセージ１（左側）
            p('<div class="bms_message bms_left">');
                p('<div class="bms_message_box">');
                    p('<div class="bms_message_content">');
                        p('<div class="bms_message_text">ほうほうこりゃー便利じゃないか</div>');
                    p('</div>');
                p('</div>');
            p('</div>');
            p('<div class="bms_clear"></div>');

            //メッセージ２（右側）
            p('<div class="bms_message bms_right">');
                p('<div class="bms_message_box">');
                    p('<div class="bms_message_content">');
                        p('<div class="bms_message_text">うん、まあまあいけとるな</div>');
                    p ('</div>');
                p('</div>');
            p('</div>');
            p('<div class="bms_clear"></div>');
        p('</div>');

        // テキストボックス、送信ボタン④
        p('<div id="bms_send">');
            p('<textarea id="bms_send_message"></textarea>');
            p('<div id="bms_send_btn">送信</div>');
        p('</div>');

        //jQueryのappend関数を使って、your_containerの直下にhtml要素を書き加える
        $('#your_container').append(chatDom.join(''));
});
