const { invoke } = window.__TAURI__.core;


let isLogin = 1;
let returnMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function checkLogin() {
  if (isLogin) {
    console.log("ログイン済みです: mypage.html");
    returnMsgEl.textContent = await invoke("logged");

  } else {
    console.log("未ログインです: login.html");
  }
  
};

document.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#loginBtn").addEventListener("click", checkLogin);
  returnMsgEl = document.querySelector("#logged-msg");

})

