import { mount } from "svelte";
import { waitLocale } from "svelte-i18n";
import "./styles/app.css";
import App from "./App.svelte";
import { setupI18n } from "./i18n/index.ts";

setupI18n();
await waitLocale();
const app = mount(App, { target: document.getElementById("app")! });
export default app;
