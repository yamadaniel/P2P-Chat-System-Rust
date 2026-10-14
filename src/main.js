const { invoke } = window.__TAURI__.core;


let isLogin = 1;
let returnMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function checkLogin() {
  const msgEl = document.querySelector("#logged-msg");

  if(!msgEl) {
    console.error("エラー: #logged-msgが見つかりません");
    return;
  }

  try {
    const result = await invoke("logged");

    msgEl.textContent = result;
    console.log("ログイン状態: ", result);

  } catch(error) {
    console.error("コマンド実行失敗: ", error);
    msgEl.textContent = "エラーが発生しました";
  }
  
};

/*
document.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#loginBtn").addEventListener("click", checkLogin);
  returnMsgEl = document.querySelector("#logged-msg");

})

*/

document.addEventListener('DOMContentLoaded',() => {
  checkLogin();
});