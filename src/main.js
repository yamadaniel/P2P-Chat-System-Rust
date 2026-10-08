const { invoke } = window.__TAURI__.core;


let isLogin = 1;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

document.addEventListener("DOMContentLoaded", function checkLogin() {
  if (isLogin) {
    console.log("ログイン済みです: mypage.html");
  } else {
    console.log("未ログインです: login.html");
  }
  
});