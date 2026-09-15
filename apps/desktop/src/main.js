import { mount } from "svelte";
import App from "./App.svelte";
import "./fc-tokens.css";
import "./app.css";

function boot() {
  const target = document.getElementById("app");
  return target ? mount(App, { target }) : null;
}

const app =
  document.readyState === "loading"
    ? (document.addEventListener("DOMContentLoaded", boot), null)
    : boot();
export default app;
